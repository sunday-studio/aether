package tag

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// GetAllTags godoc
// @Id getAllTags
// @Summary Get all tags
// @Description Returns all tags
// @Tags Tags
// @Produce json
// @Success 200 {array} db.Tag
// @Failure 500 {object} map[string]string
// @Router /tags [get]
func (t *TagsHandler) GetAllTags(c *fiber.Ctx) error {
	var tags []db.Tag
	if err := t.db.Find(&tags).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}
	return c.JSON(tags)
}
