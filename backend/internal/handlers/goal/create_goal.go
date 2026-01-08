package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"
	"time"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
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

	// Get user's current timezone from Settings (default to UTC if not found)
	var settings db.Settings
	timezone := "UTC"
	if err := h.db.First(&settings).Error; err == nil {
		timezone = settings.Timezone
		if timezone == "" {
			timezone = "UTC"
		}
	}

	// Use transaction to ensure atomicity of goal + instance creation
	return h.db.Transaction(func(tx *gorm.DB) error {
		goal := db.Goal{
			Name:               payload.Name,
			Description:        payload.Description,
			RecurrenceType:     payload.RecurrenceType,
			RecurrenceInterval: payload.RecurrenceInterval,
			RecurrenceAnchor:   payload.RecurrenceAnchor,
			RecurrenceMeta:     payload.RecurrenceMeta,
			Timezone:           timezone, // Snapshot user timezone at creation
		}

		if err := tx.Create(&goal).Error; err != nil {
			return err
		}

		// Get goal's timezone location for period calculation
		loc, err := utils.GetGoalLocation(goal.Timezone)
		if err != nil {
			return err
		}

		// Calculate first period using goal's timezone
		periodStart, periodEnd := utils.CalculateGoalPeriod(utils.RecurringGoal{
			RecurrenceType:     goal.RecurrenceType,
			RecurrenceInterval: goal.RecurrenceInterval,
			RecurrenceAnchor:   goal.RecurrenceAnchor,
		}, time.Now(), loc)

		// Create first instance with proper period calculation
		firstInstance := db.GoalInstance{
			GoalID:      goal.ID,
			PeriodStart: periodStart,
			PeriodEnd:   periodEnd,
			Status:      "active",
		}

		if err := tx.Create(&firstInstance).Error; err != nil {
			return err
		}

		// Handle tags association (outside transaction for now, but could be moved in)
		if len(payload.TagIDs) > 0 {
			var tags []db.Tag
			if err := h.db.Where("id IN ?", payload.TagIDs).Find(&tags).Error; err != nil {
				return err
			}
			if err := h.db.Model(&goal).Association("Tags").Replace(tags); err != nil {
				return err
			}
		}

		return c.JSON(goal)
	})
}
