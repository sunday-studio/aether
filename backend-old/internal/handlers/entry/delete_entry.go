package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

// DeleteEntry godoc
// @Id deleteEntry
// @Summary Delete an entry (soft delete)
// @Tags Entries
// @Produce json
// @Param id path string true "Entry ID"
// @Success 200 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /entry/{id} [delete]
func (e *EntryHandler) DeleteEntry(c *fiber.Ctx) error {
	var entry db.Entry
	if err := e.db.First(&entry, "id = ? AND is_deleted = ?", c.Params("id"), false).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "Entry not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	entry.IsDeleted = true
	if err := e.db.Save(&entry).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.SendStatus(204)
}
