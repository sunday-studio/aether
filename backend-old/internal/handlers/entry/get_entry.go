package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// GetEntries godoc
// @Id getEntries
// @Summary Get all entries
// @Description Returns all non-deleted entries, including their tags
// @Tags Entries
// @Produce json
// @Success 200 {array} db.Entry
// @Failure 500 {object} map[string]string
// @Router /entry [get]
func (e *EntryHandler) GetEntries(c *fiber.Ctx) error {
	var entries []db.Entry
	if err := e.db.
		Preload("Tags").
		Where("is_deleted = ?", false).
		Order("created_at ASC").
		Find(&entries).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(entries)
}


// GetEntryByID godoc
// @Id getEntryByID
// @Summary Get entry by ID
// @Description Returns a single entry by its ID
// @Tags Entries
// @Produce json
// @Param id path string true "Entry ID"
// @Success 200 {object} db.Entry
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /entry/{id} [get]
func (e *EntryHandler) GetEntryByID(c *fiber.Ctx) error {
	var entry db.Entry
	if err := e.db.First(&entry, "id = ? AND is_deleted = ?", c.Params("id"), false).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "Entry not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(entry)
}
