package handlers

import (
	"aether/internal/db"
	"aether/internal/logging"
	"aether/internal/utils"
	"time"

	"github.com/gofiber/fiber/v2"
)

type CreateEntryPayload struct {
	Document   string    `json:"document"`
	Date       time.Time `json:"date"`
	IsPinned   *bool     `json:"isPinned"`
	IsArchived *bool     `json:"isArchived"`
	IsDeleted  *bool     `json:"isDeleted"`
	Tags       *[]string `json:"tags"`
}

// CreateEntry godoc
// @Id createEntry
// @Summary Create a new entry
// @Description Creates a new entry with optional tags
// @Tags Entries
// @Accept json
// @Produce json
// @Param entry body handlers.CreateEntryPayload true "Entry payload"
// @Success 200 {object} db.Entry
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /entry [post]
func (e *EntryHandler) CreateEntry(c *fiber.Ctx) error {
	var payload CreateEntryPayload
	logger := logging.NewLogger()
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	entry := db.Entry{
		ID:         utils.GenerateID("entry"),
		Document:   payload.Document,
		CreatedAt:  payload.Date,
		IsPinned:   payload.IsPinned != nil && *payload.IsPinned,
		IsArchived: payload.IsArchived != nil && *payload.IsArchived,
		IsDeleted:  payload.IsDeleted != nil && *payload.IsDeleted,
	}

	logger.Info("Creating entry", "entry", entry)

	if err := e.db.Create(&entry).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	return c.JSON(entry)
}

// @Id bulkCreateEntries
// @Summary Bulk create entries
// @Tags Entries
// @Accept json
// @Produce json
// @Param entries body []CreateEntryPayload true "Entries payload"
// @Success 200 {array} db.Entry
// @Failure 400 {object} map[string]string
func (e *EntryHandler) BulkCreateEntries(c *fiber.Ctx) error {
	var payload []CreateEntryPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	for _, entry := range payload {
		entry := db.Entry{
			ID:         utils.GenerateID("entry"),
			Document:   entry.Document,
			CreatedAt:  entry.Date,
			IsPinned:   entry.IsPinned != nil && *entry.IsPinned,
			IsArchived: entry.IsArchived != nil && *entry.IsArchived,
			IsDeleted:  entry.IsDeleted != nil && *entry.IsDeleted,
		}

		if err := e.db.Create(&entry).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.JSON(payload)
}
