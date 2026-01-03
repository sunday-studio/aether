package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
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

	// Verify task exists
	var task db.Task
	if err := h.db.First(&task, "id = ?", taskID).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "task not found"})
	}

	var payload CreateSubTaskPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	if payload.Title == "" {
		return c.Status(400).JSON(fiber.Map{"error": "title is required"})
	}

	// Determine orderSort - if not provided, get the max orderSort for this task and add 1
	orderSort := 0
	if payload.OrderSort != nil {
		orderSort = *payload.OrderSort
	} else {
		var maxOrderSort int
		h.db.Model(&db.SubTask{}).
			Where("task_id = ?", taskID).
			Select("COALESCE(MAX(order_sort), -1)").
			Scan(&maxOrderSort)
		orderSort = maxOrderSort + 1
	}

	subTask := db.SubTask{
		TaskID:      taskID,
		Title:       payload.Title,
		IsCompleted: false,
		OrderSort:   orderSort,
	}

	if err := h.db.Create(&subTask).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(subTask)
}

