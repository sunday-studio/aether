package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// UpdateSubTask godoc
// @Id updateSubTask
// @Summary Update a subtask
// @Description Updates a subtask
// @Tags Tasks
// @Accept json
// @Produce json
// @Param taskId path string true "Task ID"
// @Param subtaskId path string true "Subtask ID"
// @Param subtask body handlers.UpdateSubTaskPayload true "Subtask payload"
// @Success 200 {object} db.SubTask
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{taskId}/subtasks/{subtaskId} [put]
func (h *TaskHandler) UpdateSubTask(c *fiber.Ctx) error {
	taskID := c.Params("taskId")
	subtaskID := c.Params("subtaskId")

	var payload UpdateSubTaskPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	// Verify task exists
	var task db.Task
	if err := h.db.First(&task, "id = ?", taskID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "task not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	// Find subtask
	var subtask db.SubTask
	if err := h.db.Where("id = ? AND task_id = ?", subtaskID, taskID).First(&subtask).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "subtask not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	// Update fields
	if payload.Title != nil {
		subtask.Title = *payload.Title
	}
	if payload.IsCompleted != nil {
		subtask.IsCompleted = *payload.IsCompleted
	}

	if err := h.db.Save(&subtask).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(subtask)
}
