package tag

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
)

type CreateTagPayload struct {
	Name string `json:"name"`
}

func (t *TagsHandler) CreateTag(c *fiber.Ctx) error {
	var payload CreateTagPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	tag := db.Tag{
		ID:   utils.GenerateID("tag"),
		Name: payload.Name,
	}

	if err := t.db.Create(&tag).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(payload)
}
