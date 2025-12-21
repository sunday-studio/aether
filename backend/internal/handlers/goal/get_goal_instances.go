package handlers

import (
	"aether/internal/db"
	"fmt"

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

	fmt.Println("goalID", goalID)

	var instances []db.GoalInstance
	if err := h.db.
		Preload("Tasks").
		Where("goal_id = ?", goalID).
		Order("period_start DESC").
		Find(&instances).
		Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(instances)
}
