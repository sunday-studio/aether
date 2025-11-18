package tag

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

func (t *TagsHandler) GetAllTags(c *fiber.Ctx) error {
	var tags []db.Tag
	if err := t.db.Find(&tags).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(tags)
}
