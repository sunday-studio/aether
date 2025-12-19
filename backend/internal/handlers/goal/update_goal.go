package handlers

import (
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

// UpdateGoal godoc
// @Id updateGoal
// @Summary Update a goal
// @Tags Goals
// @Accept json
// @Produce json
// @Param id path string true "Goal ID"
// @Param goal body handlers.UpdateGoalPayload true "Goal payload"
// @Success 200 {object} db.Goal
// @Failure 404 {object} map[string]string
// @Router /goals/{id} [put]
func (h *GoalHandler) UpdateGoal(c *fiber.Ctx) error {
	id := c.Params("id")

	var payload UpdateGoalPayload
	if err := c.BodyParser(&payload); err != nil {
		return c.Status(400).JSON(fiber.Map{"error": "invalid body"})
	}

	var goal db.Goal
	if err := h.db.First(&goal, "id = ?", id).Error; err != nil {
		return c.Status(404).JSON(fiber.Map{"error": "goal not found"})
	}

	if payload.Name != nil {
		goal.Name = *payload.Name
	}
	if payload.Description != nil {
		goal.Description = payload.Description
	}
	if payload.RecurrenceType != nil {
		goal.RecurrenceType = *payload.RecurrenceType
	}
	if payload.RecurrenceInterval != nil {
		goal.RecurrenceInterval = *payload.RecurrenceInterval
	}
	if payload.RecurrenceAnchor != nil {
		goal.RecurrenceAnchor = *payload.RecurrenceAnchor
	}
	if payload.RecurrenceMeta != nil {
		goal.RecurrenceMeta = *payload.RecurrenceMeta
	}

	if err := h.db.Save(&goal).Error; err != nil {
		return c.Status(500).JSON(fiber.Map{"error": err.Error()})
	}

	if payload.TagIDs != nil {
		var tags []db.Tag
		if len(*payload.TagIDs) > 0 {
			if err := h.db.Where("id IN ?", *payload.TagIDs).Find(&tags).Error; err != nil {
				return c.Status(500).JSON(fiber.Map{"error": err.Error()})
			}
		}
		if err := h.db.Model(&goal).Association("Tags").Replace(tags); err != nil {
			return c.Status(500).JSON(fiber.Map{"error": err.Error()})
		}
	}

	return c.JSON(goal)
}
