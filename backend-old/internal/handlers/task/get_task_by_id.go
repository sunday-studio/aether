package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// GetTaskByID godoc
// @Id getTaskByID
// @Summary Get task by ID
// @Tags Tasks
// @Produce json
// @Param id path string true "Task ID"
// @Success 200 {object} db.Task
// @Failure 404 {object} map[string]string
// @Router /tasks/{id} [get]
func (h *TaskHandler) GetTaskByID(c *fiber.Ctx) error {
	id := c.Params("id")

	var task db.Task
	if err := h.db.Preload("Tags").
		Preload("SubTasks", func(db *gorm.DB) *gorm.DB {
			return db.Order("order_index ASC")
		}).
		First(&task, "id = ?", id).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "task not found"})
	}

	return c.JSON(task)
}
