package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

type AddGoalToTaskPayload struct {
	GoalID string `json:"goalId"`
}

// AddGoalToTask godoc
// @Id addGoalToTask
// @Summary Add a task to the current instance of a goal
// @Tags Tasks
// @Accept json
// @Produce json
// @Param id path string true "Task ID"
// @Param goalId body handlers.AddGoalToTaskPayload true "Goal ID to add the task to"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{id}/goal [post]
func (h *TaskHandler) AddGoalToTask(c *fiber.Ctx) error {
	taskID := c.Params("id")

	if taskID == "" {
		return c.Status(400).JSON(fiber.Map{
			"error": "task ID is required",
		})
	}

	if !utils.IsValidID(taskID, "task") {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid task ID",
		})
	}

	// Get the task
	var task db.Task
	if err := h.db.Where("id = ?", taskID).First(&task).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{
				"error": "task not found",
			})
		}
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Parse goal ID from body
	var body AddGoalToTaskPayload

	if err := c.BodyParser(&body); err != nil {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid body",
		})
	}

	if body.GoalID == "" || !utils.IsValidID(body.GoalID, "goal") {
		return c.Status(400).JSON(fiber.Map{
			"error": "goal ID is required",
		})
	}

	// Get or create current goal instance (this also validates the goal exists)
	goalInstanceID, err := h.getOrCreateCurrentGoalInstance(body.GoalID)
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{
				"error": "goal not found",
			})
		}
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Update task with goal ID and goal instance ID
	task.GoalID = &body.GoalID
	task.GoalInstanceID = goalInstanceID

	if err := h.db.Save(&task).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Reload task with associations
	h.db.Preload("Tags").Preload("GoalInstance.Goal").First(&task, "id = ?", taskID)

	return c.JSON(task)
}

// RemoveGoalFromTask godoc
// @Id removeGoalFromTask
// @Summary Remove a task from its goal instance
// @Tags Tasks
// @Produce json
// @Param id path string true "Task ID"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{id}/goal [delete]
func (h *TaskHandler) RemoveGoalFromTask(c *fiber.Ctx) error {
	taskID := c.Params("id")
	if taskID == "" || !utils.IsValidID(taskID, "task") {
		return c.Status(400).JSON(fiber.Map{
			"error": "task ID is required",
		})
	}

	// Get the task
	var task db.Task
	if err := h.db.Where("id = ?", taskID).First(&task).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{
				"error": "task not found",
			})
		}
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Remove goal association (both GoalID and GoalInstanceID)
	task.GoalID = nil
	task.GoalInstanceID = nil
	if err := h.db.Save(&task).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Reload task with associations
	h.db.Preload("Tags").First(&task, "id = ?", taskID)

	return c.JSON(task)
}
