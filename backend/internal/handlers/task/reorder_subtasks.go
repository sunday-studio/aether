package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// ReorderSubTasks godoc
// @Id reorderSubTasks
// @Summary Reorder subtasks
// @Description Updates the orderSort of subtasks based on the provided ordered list
// @Tags Tasks
// @Accept json
// @Produce json
// @Param taskId path string true "Task ID"
// @Param payload body handlers.ReorderSubTasksPayload true "Reorder payload"
// @Success 200 {object} map[string]string
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Router /tasks/{taskId}/subtasks/reorder [post]
func (h *TaskHandler) ReorderSubTasks(c *fiber.Ctx) error {
	taskID := c.Params("taskId")

	// Verify task exists
	var task db.Task
	if err := h.db.First(&task, "id = ?", taskID).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "task not found"})
	}

	var payload ReorderSubTasksPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	// Update orderSort for each subtask based on its position in the array
	for index, subTaskID := range payload.SubTaskIDs {
		if err := h.db.Model(&db.SubTask{}).
			Where("id = ? AND task_id = ?", subTaskID, taskID).
			Update("order_sort", index).Error; err != nil {
			// If subtask doesn't exist or doesn't belong to this task, skip it
			continue
		}
	}

	return c.JSON(fiber.Map{"message": "subtasks reordered successfully"})
}

