package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// ReorderSubTasks godoc
// @Id reorderSubTasks
// @Summary Reorder subtasks
// @Description Reorders subtasks for a task based on the provided order
// @Tags Tasks
// @Accept json
// @Produce json
// @Param taskId path string true "Task ID"
// @Param payload body handlers.ReorderSubTasksPayload true "Reorder payload"
// @Success 200 {object} map[string]string
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{taskId}/subtasks/reorder [post]
func (h *TaskHandler) ReorderSubTasks(c *fiber.Ctx) error {
	taskID := c.Params("taskId")

	var payload ReorderSubTasksPayload
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

	// Verify all subtasks belong to this task
	var count int64
	h.db.Model(&db.SubTask{}).
		Where("task_id = ? AND id IN ?", taskID, payload.SubTaskIDs).
		Count(&count)

	if int(count) != len(payload.SubTaskIDs) {
		return c.Status(400).JSON(fiber.Map{"error": "some subtasks do not belong to this task"})
	}

	// Update order indices in a transaction
	err := h.db.Transaction(func(tx *gorm.DB) error {
		for i, subtaskID := range payload.SubTaskIDs {
			if err := tx.Model(&db.SubTask{}).
				Where("id = ? AND task_id = ?", subtaskID, taskID).
				Update("order_index", i).Error; err != nil {
				return err
			}
		}
		return nil
	})

	if err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(fiber.Map{"message": "subtasks reordered successfully"})
}
