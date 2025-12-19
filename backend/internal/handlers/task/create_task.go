package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
)

// CreateTask godoc
// @Id createTask
// @Summary Create a new task
// @Description Creates a new task (standalone or goal-based)
// @Tags Tasks
// @Accept json
// @Produce json
// @Param task body handlers.CreateTaskPayload true "Task payload"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks [post]
func (h *TaskHandler) CreateTask(c *fiber.Ctx) error {
	var payload CreateTaskPayload

	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	if payload.Title == "" {
		return c.Status(400).JSON(fiber.Map{"error": "title is required"})
	}

	task := db.Task{
		Title:          payload.Title,
		Description:    payload.Description,
		DueDate:        payload.DueDate,
		GoalInstanceID: payload.GoalInstanceID,
	}

	utils.PrettyPrint(payload)

	if err := h.db.Create(&task).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	if len(payload.TagIDs) > 0 {
		var tags []db.Tag
		if err := h.db.Where("id IN ?", payload.TagIDs).Find(&tags).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		if err := h.db.Model(&task).Association("Tags").Replace(tags); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.JSON(task)
}
