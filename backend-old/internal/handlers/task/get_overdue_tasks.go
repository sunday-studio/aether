package handlers

import (
	"aether/internal/db"
	"time"

	"github.com/gofiber/fiber/v2"
)

// GetOverdueTasks godoc
// @Id getOverdueTasks
// @Summary List overdue tasks
// @Tags Tasks
// @Produce json
// @Success 200 {array} db.Task
// @Router /tasks/overdue [get]
func (h *TaskHandler) GetOverdueTasks(c *fiber.Ctx) error {
	var tasks []db.Task

	if err := h.db.
		Where("due_date < ?", time.Now()).
		Where("is_completed = false").
		Order("due_date ASC").
		Find(&tasks).
		Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(tasks)
}
