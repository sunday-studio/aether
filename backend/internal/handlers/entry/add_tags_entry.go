package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// AddTagsToEntry godoc
// @Id addTagsToEntry
// @Summary Add tags to an entry
// @Tags Entries
// @Accept json
// @Produce json
// @Param id path string true "Entry ID"
// @Param tags body []string true "List of tag names"
// @Success 200 {object} db.Entry
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /entry/{id}/tags [post]
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

	var payload []string
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid body",
		})
	}

	// Fetch all tags to be potentially added
	var tagsToAdd []db.Tag
	if err := h.db.Where("id IN ?", payload).Find(&tagsToAdd).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Load currently assigned tags
	var existingTags []db.Tag
	if err := h.db.Model(&entry).Association("Tags").Find(&existingTags); err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Build a map of existing tag IDs for quick lookup
	existingTagIDs := make(map[string]struct{})
	for _, t := range existingTags {
		existingTagIDs[t.ID] = struct{}{}
	}

	// Filter out tags that are already assigned
	var newTags []db.Tag
	for _, tag := range tagsToAdd {
		if _, found := existingTagIDs[tag.ID]; !found {
			newTags = append(newTags, tag)
		}
	}

	// Only add new associations (skip if none are new)
	if len(newTags) > 0 {
		if err := h.db.Model(&entry).Association("Tags").Append(newTags); err != nil {
			return c.Status(500).JSON(fiber.Map{
				"error": err.Error(),
			})
		}
	}

	// Return entry with updated tags
	h.db.Preload("Tags").First(&entry)

	return c.JSON(entry)
}

// RemoveTagsFromEntry godoc
// @Id removeTagsFromEntry
// @Summary Remove a tag from an entry
// @Tags Entries
// @Accept json
// @Produce json
// @Param id path string true "Entry ID"
// @Param tagId body string true "Tag ID to remove"
// @Success 200 {object} db.Entry
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /entry/{id}/tags [delete]
func (h *EntryHandler) RemoveTagsFromEntry(c *fiber.Ctx) error {
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

	var tagID string
	if err := c.BodyParser(&tagID); err != nil || tagID == "" || !utils.IsValidID(tagID, "tag") {
		return c.Status(400).JSON(fiber.Map{
			"error": "invalid body or tag ID required",
		})
	}

	// Fetch the tag to be potentially removed
	var tagToRemove db.Tag
	if err := h.db.Where("id = ?", tagID).First(&tagToRemove).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{
				"error": "tag not found",
			})
		}
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Remove the single association
	if err := h.db.Model(&entry).Association("Tags").Delete(&tagToRemove); err != nil {
		return c.Status(500).JSON(fiber.Map{
			"error": err.Error(),
		})
	}

	// Return entry with updated tags
	h.db.Preload("Tags").First(&entry)

	return c.JSON(entry)
}
