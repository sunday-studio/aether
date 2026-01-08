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
func (h *TaskHandler) getOrCreateCurrentGoalInstance(goalID string) (*string, error) {
	// Get the goal
	var goal db.Goal
	if err := h.db.First(&goal, "id = ?", goalID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return nil, gorm.ErrRecordNotFound
		}
		return nil, err
	}

	// Calculate current period for the goal
	start, end := utils.CalculateGoalPeriod(utils.RecurringGoal{
		RecurrenceType:     goal.RecurrenceType,
		RecurrenceInterval: goal.RecurrenceInterval,
		RecurrenceAnchor:   goal.RecurrenceAnchor,
	}, time.Now())

	// Get or create current goal instance
	var instance db.GoalInstance
	err := h.db.
		Where("goal_id = ? AND period_start = ?", goal.ID, start).
		First(&instance).
		Error

	if err == gorm.ErrRecordNotFound {
		// Create new instance
		instance = db.GoalInstance{
			GoalID:      goal.ID,
			PeriodStart: start,
			PeriodEnd:   end,
			Status:      "active",
		}
		if err := h.db.Create(&instance).Error; err != nil {
			return nil, err
		}
	} else if err != nil {
		return nil, err
	}

	return &instance.ID, nil
}
