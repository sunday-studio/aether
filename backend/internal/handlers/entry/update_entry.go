package handlers

import (
	"aether/internal/db"
	"fmt"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// UpdateEntry godoc
// @Id updateEntry
// @Summary Update an entry
// @Tags Entries
// @Accept json
// @Produce json
// @Param id path string true "Entry ID"
// @Param entry body handlers.CreateEntryPayload true "Updated entry payload"
// @Success 200 {object} db.Entry
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /entry/{id} [put]
func (e *EntryHandler) UpdateEntry(c *fiber.Ctx) error {
	var entry db.Entry
	if err := e.db.First(&entry, "id = ? AND is_deleted = ?", c.Params("id"), false).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "Entry not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	var payload db.Entry
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	fmt.Println("payload ->", payload, entry.ID)

	entry.Document = payload.Document
	entry.IsPinned = payload.IsPinned
	entry.IsArchived = payload.IsArchived
	entry.IsDeleted = payload.IsDeleted

	if err := e.db.Save(&entry).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(entry)
}
