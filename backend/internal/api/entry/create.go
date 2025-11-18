package entry

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func CreateEntry(c *fiber.Ctx, gormDB *gorm.DB) error {
	var payload db.Entry
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	if err := gormDB.Create(&payload).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(payload)
}
