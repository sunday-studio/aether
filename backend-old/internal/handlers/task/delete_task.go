package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// DeleteTask godoc
// @Summary Delete a task (soft delete)
// @Id deleteTaskById
// @Description Marks the specified task as deleted (soft-delete).
// @Tags Tasks
// @Accept json
// @Produce json
// @Param id path string true "Task ID"
// @Success 204 {object} nil
// @Failure 404 {object} map[string]string
// @Router /tasks/{id} [delete]
func (h *TaskHandler) DeleteTask(c *fiber.Ctx) error {
	id := c.Params("id")

	if err := h.db.Delete(&db.Task{}, "id = ?", id).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "task not found"})
	}

	return c.SendStatus(204)
}
