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

	var goal db.Goal
	if err := h.db.First(&goal, "id = ?", goalID).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "goal not found"})
	}

	start, end := utils.CalculateGoalPeriod(utils.RecurringGoal{
		RecurrenceType:     goal.RecurrenceType,
		RecurrenceInterval: goal.RecurrenceInterval,
		RecurrenceAnchor:   goal.RecurrenceAnchor,
	}, time.Now())

	var instance db.GoalInstance
	err := h.db.
		Where("goal_id = ? AND period_start = ?", goal.ID, start).
		First(&instance).
		Error

	if err == gorm.ErrRecordNotFound {
		instance = db.GoalInstance{
			GoalID:      goal.ID,
			PeriodStart: start,
			PeriodEnd:   end,
			Status:      "active",
		}
		if err := h.db.Create(&instance).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	} else if err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(instance)
}
