package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"
	"fmt"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// AddTagsToTask godoc
// @Id addTagsToTask
// @Summary Add tags to a task
// @Tags Tasks
// @Accept json
// @Produce json
// @Param id path string true "Task ID"
// @Param tags body []string true "List of tag IDs to add"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{id}/tags [post]
func (h *TaskHandler) AddTagsToTask(c *fiber.Ctx) error {
	taskID := c.Params("id")
	fmt.Println("taskID AddTagsToTask ->", taskID)

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

	var tagIDs []string
	if err := c.BodyParser(&tagIDs); err != nil {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid body",
		})
	}

	var tags []db.Tag
	if len(tagIDs) > 0 {
		if err := h.db.Where("id IN ?", tagIDs).Find(&tags).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	}

	if len(tags) > 0 {
		if err := h.db.Model(&task).Association("Tags").Append(&tags); err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	}
	h.db.Preload("Tags").First(&task)

	return c.JSON(task)
}

// RemoveTagsFromTask godoc
// @Id removeTagsFromTask
// @Summary Remove tags from a task
// @Tags Tasks
// @Accept json
// @Produce json
// @Param id path string true "Task ID"
// @Param tags body []string true "List of tag IDs to remove"
// @Success 200 {object} db.Task
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tasks/{id}/tags [delete]
func (h *TaskHandler) RemoveTagsFromTask(c *fiber.Ctx) error {
	taskID := c.Params("id")
	if taskID == "" || !utils.IsValidID(taskID, "task") {
		return c.Status(400).JSON(fiber.Map{
			"error": "task ID is required",
		})
	}

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

	var tagIDs []string
	if err := c.BodyParser(&tagIDs); err != nil {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid body",
		})
	}

	var tags []db.Tag
	if len(tagIDs) > 0 {
		if err := h.db.Where("id IN ?", tagIDs).Find(&tags).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	}

	if len(tags) > 0 {
		if err := h.db.Model(&task).Association("Tags").Delete(tags); err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	}

	h.db.Preload("Tags").First(&task)

	return c.JSON(task)
}
