package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"
	"time"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// CreateTask godoc
// @Id createTask
// @Summary Create a new task
// @Description Creates a new task (standalone or goal-based)
// @Tags Tasks
// @Accept json
// @Produce json
// @Param task body handlers.CreateTaskPayload true "Task payload"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks [post]
func (h *TaskHandler) CreateTask(c *fiber.Ctx) error {
	var payload CreateTaskPayload

	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	if payload.Title == "" {
		return c.Status(400).JSON(fiber.Map{"error": "title is required"})
	}

	task := db.Task{
		Title:       payload.Title,
		Description: payload.Description,
		DueDate:     payload.DueDate,
		GoalID:      payload.GoalID,
	}

	// If goalId is provided, find or create the current goal instance
	if payload.GoalID != nil {
		goalInstanceID, err := h.getOrCreateCurrentGoalInstance(*payload.GoalID)
		if err != nil {
			if err == gorm.ErrRecordNotFound {
				return c.Status(404).JSON(fiber.Map{"error": "goal not found"})
			}
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		task.GoalInstanceID = goalInstanceID
	}

	if err := h.db.Create(&task).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	if len(payload.TagIDs) > 0 {
		var tags []db.Tag
		if err := h.db.Where("id IN ?", payload.TagIDs).Find(&tags).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		if err := h.db.Model(&task).Association("Tags").Replace(tags); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.JSON(task)
}

// getOrCreateCurrentGoalInstance finds or creates the current goal instance for a given goalId
// Uses transaction to ensure atomicity and last-instance strategy for optimal performance
func (h *TaskHandler) getOrCreateCurrentGoalInstance(goalID string) (*string, error) {
	var instanceID *string

	// Use transaction to ensure task and instance creation are atomic
	err := h.db.Transaction(func(tx *gorm.DB) error {
		// Get goal (only fetch needed fields for optimization)
		var goal db.Goal
		if err := tx.Select("id", "recurrence_type", "recurrence_interval", "recurrence_anchor", "timezone").
			First(&goal, "id = ?", goalID).Error; err != nil {
			if err == gorm.ErrRecordNotFound {
				return gorm.ErrRecordNotFound
			}
			return err
		}

		// Get last created instance using optimized query
		lastInstance, err := h.getLastGoalInstanceWithTx(tx, goalID)
		if err != nil {
			return err
		}

		// If no instance exists, create the first one
		if lastInstance == nil {
			// Get goal's timezone location
			loc, err := utils.GetGoalLocation(goal.Timezone)
			if err != nil {
				return err
			}

			// Calculate first period (lazy calculation - only when creating)
			periodStart, periodEnd := utils.CalculateGoalPeriod(utils.RecurringGoal{
				RecurrenceType:     goal.RecurrenceType,
				RecurrenceInterval: goal.RecurrenceInterval,
				RecurrenceAnchor:   goal.RecurrenceAnchor,
			}, time.Now(), loc)

			instance := db.GoalInstance{
				GoalID:      goal.ID,
				PeriodStart: periodStart,
				PeriodEnd:   periodEnd,
				Status:      "active",
			}

			if err := tx.Create(&instance).Error; err != nil {
				return err
			}

			instanceID = &instance.ID
			return nil
		}

		// Check if new instance should be created
		shouldCreate, err := h.checkCreateNewGoalInstanceWithTx(tx, goal, lastInstance)
		if err != nil {
			return err
		}

		if shouldCreate {
			// Get goal's timezone location
			loc, err := utils.GetGoalLocation(goal.Timezone)
			if err != nil {
				return err
			}

			// Calculate new period (lazy calculation - only when creating)
			periodStart, periodEnd := utils.CalculateGoalPeriod(utils.RecurringGoal{
				RecurrenceType:     goal.RecurrenceType,
				RecurrenceInterval: goal.RecurrenceInterval,
				RecurrenceAnchor:   goal.RecurrenceAnchor,
			}, time.Now(), loc)

			instance := db.GoalInstance{
				GoalID:      goal.ID,
				PeriodStart: periodStart,
				PeriodEnd:   periodEnd,
				Status:      "active",
			}

			if err := tx.Create(&instance).Error; err != nil {
				return err
			}

			instanceID = &instance.ID
			return nil
		}

		// Use last instance (no new instance needed)
		instanceID = &lastInstance.ID
		return nil
	})

	if err != nil {
		return nil, err
	}

	return instanceID, nil
}

// getLastGoalInstanceWithTx retrieves the last goal instance within a transaction
func (h *TaskHandler) getLastGoalInstanceWithTx(tx *gorm.DB, goalID string) (*db.GoalInstance, error) {
	var instance db.GoalInstance

	err := tx.
		Select("id", "created_at", "period_start", "period_end", "status").
		Where("goal_id = ?", goalID).
		Order("created_at DESC").
		Limit(1).
		First(&instance).
		Error

	if err == gorm.ErrRecordNotFound {
		return nil, nil // No instance found, not an error
	}

	if err != nil {
		return nil, err
	}

	return &instance, nil
}

// checkCreateNewGoalInstanceWithTx checks if new instance should be created within a transaction
func (h *TaskHandler) checkCreateNewGoalInstanceWithTx(tx *gorm.DB, goal db.Goal, lastInstance *db.GoalInstance) (bool, error) {
	// If no last instance exists, we should create the first one
	if lastInstance == nil {
		return true, nil
	}

	// Get goal's timezone location
	loc, err := utils.GetGoalLocation(goal.Timezone)
	if err != nil {
		return false, err
	}

	// Convert current time and last instance date to goal's timezone
	nowInTZ := utils.NowInTimezone(loc)
	lastInstanceDateInTZ := lastInstance.CreatedAt.In(loc)

	// Calculate calendar days since last instance (DST-safe)
	daysSince := utils.DaysSinceInTimezone(lastInstanceDateInTZ, nowInTZ, loc)

	// Get the interval (all intervals are in days)
	interval := goal.RecurrenceInterval
	if interval <= 0 {
		interval = 1
	}

	// For all recurrence types, check if days since last instance >= interval
	if daysSince >= interval {
		return true, nil
	}

	return false, nil
}
