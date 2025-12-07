package tag

import (
	"aether/internal/db"
	"aether/internal/utils"

	"github.com/gofiber/fiber/v2"
)

type CreateTagPayload struct {
	Name string `json:"name"`
}

// CreateTag godoc
// @Id createTag
// @Summary Create a new tag
// @Tags Tags
// @Accept json
// @Produce json
// @Param tag body CreateTagPayload true "Tag payload"
// @Success 200 {object} db.Tag
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tags [post]
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

	return c.JSON(tag)
}


// BulkCreateTags godoc
// @Id bulkCreateTags
// @Summary Bulk create tags
// @Tags Tags
// @Accept json
// @Produce json
// @Param tags body []CreateTagPayload true "Tags payload"
// @Success 200 {array} db.Tag
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /tags/bulk-create [post]
func (t *TagsHandler) BulkCreateTags(c *fiber.Ctx) error {
	var payload []CreateTagPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	var tags []db.Tag
	for _, tagPayload := range payload {
		tag := db.Tag{
			ID:   utils.GenerateID("tag"),
			Name: tagPayload.Name,
		}

		if err := t.db.Create(&tag).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		tags = append(tags, tag)
	}

	return c.JSON(tags)
}