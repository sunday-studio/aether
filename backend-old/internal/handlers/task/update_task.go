package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// UpdateTask godoc
// @Id updateTask
// @Summary Update a task
// @Tags Tasks
// @Accept json
// @Produce json
// @Param id path string true "Task ID"
// @Param task body handlers.UpdateTaskPayload true "Task payload"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 409 {object} map[string]interface{} "Conflict: Task was modified by another device"
// @Failure 500 {object} map[string]string
// @Router /tasks/{id} [put]
func (h *TaskHandler) UpdateTask(c *fiber.Ctx) error {
	id := c.Params("id")

	var payload UpdateTaskPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	var task db.Task
	if err := h.db.First(&task, "id = ?", id).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "task not found"})
	}

	// Last-Write-Wins: Check if client's UpdatedAt is older than server's
	if payload.UpdatedAt != nil && !payload.UpdatedAt.IsZero() {
		if payload.UpdatedAt.Before(task.UpdatedAt) {
			// Client has stale data, return current server version
			return c.Status(409).JSON(fiber.Map{
				"error":   "conflict",
				"message": "Task was modified by another device",
				"current": task,
			})
		}
	}

	if payload.Title != nil {
		task.Title = *payload.Title
	}
	if payload.Description != nil {
		task.Description = payload.Description
	}
	if payload.DueDate != nil {
		task.DueDate = payload.DueDate
	} else if payload.DueDate == nil {
		task.DueDate = nil
	}

	if payload.IsCompleted != nil {
		task.IsCompleted = *payload.IsCompleted
	}
	if payload.GoalID != nil {
		task.GoalID = payload.GoalID
		// Find or create the current goal instance for the goal
		goalInstanceID, err := h.getOrCreateCurrentGoalInstance(*payload.GoalID)
		if err != nil {
			if err == gorm.ErrRecordNotFound {
				return c.Status(404).JSON(fiber.Map{"error": "goal not found"})
			}
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		task.GoalInstanceID = goalInstanceID
	}

	if err := h.db.Save(&task).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	if payload.TagIDs != nil {
		var tags []db.Tag
		if len(*payload.TagIDs) > 0 {
			if err := h.db.Where("id IN ?", *payload.TagIDs).Find(&tags).Error; err != nil {
				return c.Status(500).JSON(fiber.Map{"error": err.Error()})
			}
		}
		if err := h.db.Model(&task).Association("Tags").Replace(tags); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.JSON(task)
}
