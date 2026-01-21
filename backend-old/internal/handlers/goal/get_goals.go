package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// GetGoals godoc
// @Id getGoals
// @Summary List active goals
// @Tags Goals
// @Produce json
// @Success 200 {array} db.Goal
// @Router /goals [get]
func (h *GoalHandler) GetGoals(c *fiber.Ctx) error {
	var goals []db.Goal
	if err := h.db.Find(&goals).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(goals)
}
