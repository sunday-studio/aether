package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// DeleteSubTask godoc
// @Id deleteSubTask
// @Summary Delete a subtask (soft delete)
// @Description Marks the specified subtask as deleted (soft-delete)
// @Tags Tasks
// @Accept json
// @Produce json
// @Param taskId path string true "Task ID"
// @Param subtaskId path string true "Subtask ID"
// @Success 204 {object} nil
// @Failure 404 {object} map[string]string
// @Router /tasks/{taskId}/subtasks/{subtaskId} [delete]
func (h *TaskHandler) DeleteSubTask(c *fiber.Ctx) error {
	taskID := c.Params("taskId")
	subtaskID := c.Params("subtaskId")

	// Verify task exists
	var task db.Task
	if err := h.db.First(&task, "id = ?", taskID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "task not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	// Delete subtask (soft delete)
	if err := h.db.Where("id = ? AND task_id = ?", subtaskID, taskID).Delete(&db.SubTask{}).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "subtask not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.SendStatus(204)
}
