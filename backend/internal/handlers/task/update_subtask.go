package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// UpdateSubTask godoc
// @Id updateSubTask
// @Summary Update a subtask
// @Tags Tasks
// @Accept json
// @Produce json
// @Param taskId path string true "Task ID"
// @Param subtaskId path string true "Subtask ID"
// @Param subtask body handlers.UpdateSubTaskPayload true "Subtask payload"
// @Success 200 {object} db.SubTask
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Router /tasks/{taskId}/subtasks/{subtaskId} [put]
func (h *TaskHandler) UpdateSubTask(c *fiber.Ctx) error {
	taskID := c.Params("taskId")
	subTaskID := c.Params("subtaskId")

	var payload UpdateSubTaskPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	var subTask db.SubTask
	if err := h.db.Where("id = ? AND task_id = ?", subTaskID, taskID).First(&subTask).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "subtask not found"})
	}

	if payload.Title != nil {
		subTask.Title = *payload.Title
	}
	if payload.IsCompleted != nil {
		subTask.IsCompleted = *payload.IsCompleted
	}
	if payload.OrderSort != nil {
		subTask.OrderSort = *payload.OrderSort
	}

	if err := h.db.Save(&subTask).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(subTask)
}

