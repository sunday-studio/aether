package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// GetTrashedTasks godoc
// @Id getTrashedTasks
// @Summary List deleted tasks
// @Tags Trash
// @Produce json
// @Success 200 {array} db.Task
// @Router /trash/tasks [get]
func (h *TrashHandler) GetTrashedTasks(c *fiber.Ctx) error {
	var tasks []db.Task

	if err := h.db.
		Unscoped().
		Where("deleted_at IS NOT NULL").
		Order("deleted_at DESC").
		Find(&tasks).
		Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(tasks)
}
