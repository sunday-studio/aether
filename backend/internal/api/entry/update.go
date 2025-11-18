package entry

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func UpdateEntry(c *fiber.Ctx, gormDB *gorm.DB) error {
	var entry db.Entry
	if err := gormDB.First(&entry, "id = ? AND is_deleted = ?", c.Params("id"), false).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "Entry not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	var payload db.Entry
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	entry.Document = payload.Document
	entry.IsPinned = payload.IsPinned
	entry.IsArchived = payload.IsArchived
	entry.IsDeleted = payload.IsDeleted

	if err := gormDB.Save(&entry).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(entry)
}
