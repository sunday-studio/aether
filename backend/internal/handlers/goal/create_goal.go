package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// CreateGoal godoc
// @Id createGoal
// @Summary Create a new goal
// @Tags Goals
// @Accept json
// @Produce json
// @Param goal body handlers.CreateGoalPayload true "Goal payload"
// @Success 200 {object} db.Goal
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /goals [post]
func (h *GoalHandler) CreateGoal(c *fiber.Ctx) error {
	var payload CreateGoalPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	if payload.Name == "" || payload.RecurrenceType == "" {
		return c.Status(400).JSON(fiber.Map{"error": "missing required fields"})
	}

	goal := db.Goal{
		Name:               payload.Name,
		Description:        payload.Description,
		RecurrenceType:     payload.RecurrenceType,
		RecurrenceInterval: payload.RecurrenceInterval,
		RecurrenceAnchor:   payload.RecurrenceAnchor,
		RecurrenceMeta:     payload.RecurrenceMeta,
	}

	if err := h.db.Create(&goal).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	if len(payload.TagIDs) > 0 {
		var tags []db.Tag
		if err := h.db.Where("id IN ?", payload.TagIDs).Find(&tags).Error; err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
		if err := h.db.Model(&goal).Association("Tags").Replace(tags); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.JSON(goal)
}
