package entry

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func GetEntries(c *fiber.Ctx, gormDB *gorm.DB) error {
	var entries []db.Entry
	if err := gormDB.Where("is_deleted = ?", false).Find(&entries).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(entries)
}

func GetEntryByID(c *fiber.Ctx, gormDB *gorm.DB) error {
	var entry db.Entry
	if err := gormDB.First(&entry, "id = ? AND is_deleted = ?", c.Params("id"), false).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return c.Status(404).JSON(fiber.Map{"error": "Entry not found"})
		}
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(entry)
}

func GetEntryByDateRange(c *fiber.Ctx, gormDB *gorm.DB) error {
	var entries []db.Entry
	if err := gormDB.Where("created_at BETWEEN ? AND ?", c.Query("startDate"), c.Query("endDate")).Find(&entries).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(entries)
}
