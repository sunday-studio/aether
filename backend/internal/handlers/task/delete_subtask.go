package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// DeleteSubTask godoc
// @Id deleteSubTask
// @Summary Delete a subtask (soft delete)
// @Tags Tasks
// @Param taskId path string true "Task ID"
// @Param subtaskId path string true "Subtask ID"
// @Success 204
// @Failure 404 {object} map[string]string
// @Router /tasks/{taskId}/subtasks/{subtaskId} [delete]
func (h *TaskHandler) DeleteSubTask(c *fiber.Ctx) error {
	taskID := c.Params("taskId")
	subTaskID := c.Params("subtaskId")

	if err := h.db.Where("id = ? AND task_id = ?", subTaskID, taskID).Delete(&db.SubTask{}).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "subtask not found"})
	}

	return c.SendStatus(204)
}

