package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// RestoreTask godoc
// @Id restoreTask
// @Summary Restore a deleted task
// @Tags Trash
// @Param id path string true "Task ID"
// @Success 204
// @Failure 500 {object} map[string]string
// @Router /trash/{id}/restore [post]
func (h *TrashHandler) RestoreTask(c *fiber.Ctx) error {
	id := c.Params("id")

	if err := h.db.
		Unscoped().
		Model(&db.Task{}).
		Where("id = ?", id).
		Update("deleted_at", nil).
		Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.SendStatus(204)
}
