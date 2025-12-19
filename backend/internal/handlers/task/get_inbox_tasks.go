package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// GetInboxTasks godoc
// @Id getInboxTasks
// @Summary List standalone tasks
// @Description Tasks not attached to any goal
// @Tags Tasks
// @Produce json
// @Success 200 {array} db.Task
// @Router /tasks/inbox [get]
func (h *TaskHandler) GetInboxTasks(c *fiber.Ctx) error {
	var tasks []db.Task

	if err := h.db.
		Preload("Tags").
		Where("goal_instance_id IS NULL").
		Order("due_date ASC").
		Find(&tasks).
		Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(tasks)
}
