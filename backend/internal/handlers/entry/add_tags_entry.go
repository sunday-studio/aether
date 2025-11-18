package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func (h *EntryHandler) AddTagsToEntry(c *fiber.Ctx) error {
	entryID := c.Params("id")
	if entryID == "" || !utils.IsValidID(entryID, "entry") {
		return c.Status(400).JSON(fiber.Map{
			"error": "entry ID is required",
		})
	}

	var entry db.Entry
	if err := h.db.
		Where("id = ? AND is_deleted = ?", entryID, false).
		First(&entry).Error; err != nil {

		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{
				"error": "entry not found",
			})
		}

		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	var payload struct {
		Tags []string `json:"tags"`
	}

	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid request body",
		})
	}

	var tags []db.Tag
	if len(payload.Tags) > 0 {
		if err := h.db.Where("id IN ?", payload.Tags).Find(&tags).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	}

	if err := h.db.Model(&entry).Association("Tags").Replace(tags); err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Return entry with updated tags
	h.db.Preload("Tags").First(&entry)

	return c.JSON(entry)
}
