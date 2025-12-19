package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// GetGoalInstances godoc
// @Id getGoalInstances
// @Summary List instances for a goal
// @Tags GoalInstances
// @Produce json
// @Param goalId path string true "Goal ID"
// @Success 200 {array} db.GoalInstance
// @Router /goals/{goalId}/instances [get]
func (h *GoalHandler) GetGoalInstances(c *fiber.Ctx) error {
	goalID := c.Params("goalId")

	var instances []db.GoalInstance
	if err := h.db.
		Where("goal_id = ?", goalID).
		Order("period_start DESC").
		Find(&instances).
		Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(instances)
}
