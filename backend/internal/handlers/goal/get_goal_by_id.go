package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// GetGoalByID godoc
// @Id getGoalByID
// @Summary Get goal by ID
// @Tags Goals
// @Produce json
// @Param id path string true "Goal ID"
// @Success 200 {object} db.Goal
// @Failure 404 {object} map[string]string
// @Router /goals/{id} [get]
func (h *GoalHandler) GetGoalByID(c *fiber.Ctx) error {
	id := c.Params("id")
	var goal db.Goal
	if err := h.db.First(&goal, "id = ?", id).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "goal not found"})
	}
	return c.JSON(goal)
}
