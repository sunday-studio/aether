package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"
	"fmt"

	"github.com/gofiber/fiber/v2"
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
// @Router /tasks/{id} [put]
func (h *TaskHandler) UpdateTask(c *fiber.Ctx) error {
	id := c.Params("id")

	fmt.Println("Updating task", id)
	utils.PrettyPrint(c.Body())

	var payload UpdateTaskPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	utils.PrettyPrint(payload)

	var task db.Task
	if err := h.db.First(&task, "id = ?", id).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "task not found"})
	}

	if payload.Title != nil {
		task.Title = *payload.Title
	}
	if payload.Description != nil {
		task.Description = payload.Description
	}
	if payload.DueDate == nil {
		task.DueDate = nil
	}
	if payload.IsCompleted != nil {
		task.IsCompleted = *payload.IsCompleted
	}
	if payload.GoalInstanceID != nil {
		task.GoalInstanceID = payload.GoalInstanceID
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
