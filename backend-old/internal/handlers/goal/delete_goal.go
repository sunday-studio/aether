package handlers

import (
	"aether/internal/db"
	"time"

	"github.com/gofiber/fiber/v2"
)

// DeleteGoal godoc
// @Id deleteGoal
// @Summary Delete a goal (soft delete)
// @Tags Goals
// @Param id path string true "Goal ID"
// @Success 204
// @Router /goals/{id} [delete]
func (h *GoalHandler) DeleteGoal(c *fiber.Ctx) error {
	id := c.Params("id")

	// soft delete goal
	if err := h.db.Delete(&db.Goal{}, "id = ?", id).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	// soft delete instances
	h.db.
		Model(&db.GoalInstance{}).
		Where("goal_id = ?", id).
		Update("deleted_at", time.Now())

	return c.SendStatus(204)
}
