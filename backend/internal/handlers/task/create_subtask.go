package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// CreateSubTask godoc
// @Id createSubTask
// @Summary Create a new subtask
// @Description Creates a new subtask for a task
// @Tags Tasks
// @Accept json
// @Produce json
// @Param taskId path string true "Task ID"
// @Param subtask body handlers.CreateSubTaskPayload true "Subtask payload"
// @Success 200 {object} db.SubTask
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{taskId}/subtasks [post]
func (h *TaskHandler) CreateSubTask(c *fiber.Ctx) error {
	taskID := c.Params("taskId")

	var payload CreateSubTaskPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	if payload.Title == "" {
		return c.Status(400).JSON(fiber.Map{"error": "title is required"})
	}

	// Verify task exists
	var task db.Task
	if err := h.db.First(&task, "id = ?", taskID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "task not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	// Get the maximum order index for subtasks of this task
	var maxOrder int
	h.db.Model(&db.SubTask{}).
		Where("task_id = ?", taskID).
		Select("COALESCE(MAX(order_index), -1)").
		Scan(&maxOrder)

	subtask := db.SubTask{
		Title:      payload.Title,
		TaskID:     taskID,
		OrderIndex: maxOrder + 1,
	}

	if err := h.db.Create(&subtask).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(subtask)
}
