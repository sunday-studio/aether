package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// GetSubTasks godoc
// @Id getSubTasks
// @Summary Get all subtasks for a task
// @Description Gets all subtasks for a specific task
// @Tags Tasks
// @Produce json
// @Param taskId path string true "Task ID"
// @Success 200 {array} db.SubTask
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{taskId}/subtasks [get]
func (h *TaskHandler) GetSubTasks(c *fiber.Ctx) error {
	taskID := c.Params("taskId")

	// Verify task exists
	var task db.Task
	if err := h.db.First(&task, "id = ?", taskID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "task not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	// Get all subtasks for this task
	var subtasks []db.SubTask
	if err := h.db.Where("task_id = ?", taskID).
		Order("order_index ASC").
		Find(&subtasks).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(subtasks)
}
