package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"
	"time"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// AddGoalToTask godoc
// @Id addGoalToTask
// @Summary Add a task to the current instance of a goal
// @Tags Tasks
// @Accept json
// @Produce json
// @Param id path string true "Task ID"
// @Param goalId body string true "Goal ID to add the task to"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{id}/goal [post]
func (h *TaskHandler) AddGoalToTask(c *fiber.Ctx) error {
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

	// Parse goal ID from body
	var body struct {
		GoalID string `json:"goalId"`
	}

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

	// Get the goal
	var goal db.Goal
	if err := h.db.First(&goal, "id = ?", body.GoalID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{
				"error": "goal not found",
			})
		}
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Calculate current period for the goal
	start, end := utils.CalculateGoalPeriod(utils.RecurringGoal{
		RecurrenceType:     goal.RecurrenceType,
		RecurrenceInterval: goal.RecurrenceInterval,
		RecurrenceAnchor:   goal.RecurrenceAnchor,
	}, time.Now())

	// Get or create current goal instance
	var instance db.GoalInstance
	err := h.db.
		Where("goal_id = ? AND period_start = ?", goal.ID, start).
		First(&instance).
		Error

	if err == gorm.ErrRecordNotFound {
		// Create new instance
		instance = db.GoalInstance{
			GoalID:      goal.ID,
			PeriodStart: start,
			PeriodEnd:   end,
			Status:      "active",
		}
		if err := h.db.Create(&instance).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	} else if err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Update task with goal instance ID
	task.GoalInstanceID = &instance.ID
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

	// Remove goal instance association
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
