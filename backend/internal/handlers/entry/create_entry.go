package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
)

type CreateEntryPayload struct {
	Document   string    `json:"document"`
	IsPinned   *bool     `json:"isPinned"`
	IsArchived *bool     `json:"isArchived"`
	IsDeleted  *bool     `json:"isDeleted"`
	Tags       *[]string `json:"tags"`
}

func (e *EntryHandler) CreateEntry(c *fiber.Ctx) error {
	var payload CreateEntryPayload

	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	entry := db.Entry{
		ID:         utils.GenerateID("entry"),
		Document:   payload.Document,
		IsPinned:   *payload.IsPinned,
		IsArchived: *payload.IsArchived,
		IsDeleted:  *payload.IsDeleted,
		// figure out the best way to handle this
		// Tags:       &[]db.Tag{},
	}

	if err := e.db.Create(&entry).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(payload)
}
