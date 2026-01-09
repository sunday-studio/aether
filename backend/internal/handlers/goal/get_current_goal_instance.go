package handlers

import (
	"time"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"

	"aether/internal/db"
	"aether/internal/utils"
)

// GetCurrentGoalInstance godoc
// @Id getCurrentGoalInstance
// @Summary Get or create current goal instance
// @Tags GoalInstances
// @Produce json
// @Param goalId path string true "Goal ID"
// @Success 200 {object} db.GoalInstance
// @Router /goals/{goalId}/instances/current [get]
func (h *GoalHandler) GetCurrentGoalInstance(c *fiber.Ctx) error {
	goalID := c.Params("goalId")

	// Use transaction to prevent race conditions
	return h.db.Transaction(func(tx *gorm.DB) error {
		// Get goal (fetch is_non_recurring field as well)
		var goal db.Goal
		if err := tx.Select("id", "is_non_recurring", "recurrence_type", "recurrence_interval", "recurrence_anchor", "timezone").
			First(&goal, "id = ?", goalID).Error; err != nil {
			if err == gorm.ErrRecordNotFound {
				return c.Status(404).JSON(fiber.Map{"error": "goal not found"})
			}
			return err
		}

		// Get last created instance using optimized query
		lastInstance, err := h.getLastGoalInstanceWithTx(tx, goalID)
		if err != nil {
			return err
		}

		// For non-recurring goals, always return the same instance (or create if none exists)
		if goal.IsNonRecurring {
			if lastInstance == nil {
				// Create instance for non-recurring goal
				instance := db.GoalInstance{
					GoalID:      goal.ID,
					PeriodStart: time.Now(),
					PeriodEnd:   nil, // nil for non-recurring goals
					Status:      "active",
				}

				if err := tx.Create(&instance).Error; err != nil {
					return err
				}

				return c.JSON(instance)
			}

			// Return existing instance (non-recurring goals always use the same instance)
			return c.JSON(lastInstance)
		}

		// For recurring goals, use existing logic
		// If no instance exists, create the first one
		if lastInstance == nil {
			// Get goal's timezone location
			loc, err := utils.GetGoalLocation(goal.Timezone)
			if err != nil {
				return err
			}

			// Calculate first period (lazy calculation - only when creating)
			periodStart, periodEnd := utils.CalculateGoalPeriod(utils.RecurringGoal{
				RecurrenceType:     *goal.RecurrenceType,
				RecurrenceInterval: *goal.RecurrenceInterval,
				RecurrenceAnchor:   *goal.RecurrenceAnchor,
			}, time.Now(), loc)

			instance := db.GoalInstance{
				GoalID:      goal.ID,
				PeriodStart: periodStart,
				PeriodEnd:   &periodEnd,
				Status:      "active",
			}

			if err := tx.Create(&instance).Error; err != nil {
				return err
			}

			return c.JSON(instance)
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
				RecurrenceType:     *goal.RecurrenceType,
				RecurrenceInterval: *goal.RecurrenceInterval,
				RecurrenceAnchor:   *goal.RecurrenceAnchor,
			}, time.Now(), loc)

			instance := db.GoalInstance{
				GoalID:      goal.ID,
				PeriodStart: periodStart,
				PeriodEnd:   &periodEnd,
				Status:      "active",
			}

			if err := tx.Create(&instance).Error; err != nil {
				return err
			}

			return c.JSON(instance)
		}

		// Return last instance (no new instance needed)
		return c.JSON(lastInstance)
	})
}

// getLastGoalInstanceWithTx retrieves the last goal instance within a transaction
func (h *GoalHandler) getLastGoalInstanceWithTx(tx *gorm.DB, goalID string) (*db.GoalInstance, error) {
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
func (h *GoalHandler) checkCreateNewGoalInstanceWithTx(tx *gorm.DB, goal db.Goal, lastInstance *db.GoalInstance) (bool, error) {
	// Non-recurring goals never create new instances - always use the same one
	if goal.IsNonRecurring {
		return false, nil
	}

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
	interval := 1
	if goal.RecurrenceInterval != nil {
		interval = *goal.RecurrenceInterval
	}
	if interval <= 0 {
		interval = 1
	}

	// For all recurrence types, check if days since last instance >= interval
	if daysSince >= interval {
		return true, nil
	}

	return false, nil
}
