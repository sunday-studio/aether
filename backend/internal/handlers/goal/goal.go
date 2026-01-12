package handlers

import (
	"aether/internal/db"
	"time"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type CreateGoalPayload struct {
	Name               string         `json:"name"`
	Description        *string        `json:"description"`
	IsNonRecurring     *bool          `json:"isNonRecurring"`     // optional, defaults to false
	RecurrenceType     *string        `json:"recurrenceType"`     // nullable for non-recurring goals
	RecurrenceInterval *int           `json:"recurrenceInterval"` // nullable for non-recurring goals
	RecurrenceAnchor   *time.Time     `json:"recurrenceAnchor"`   // nullable for non-recurring goals
	RecurrenceMeta     datatypes.JSON `json:"recurrenceMeta"`
	TagIDs             []string       `json:"tagIds"`
}

type UpdateGoalPayload struct {
	Name               *string         `json:"name"`
	Description        *string         `json:"description"`
	IsNonRecurring     *bool           `json:"isNonRecurring"` // optional, but if provided must match current value
	RecurrenceType     *string         `json:"recurrenceType"`
	RecurrenceInterval *int            `json:"recurrenceInterval"`
	RecurrenceAnchor   *time.Time      `json:"recurrenceAnchor"`
	RecurrenceMeta     *datatypes.JSON `json:"recurrenceMeta"`
	TagIDs             *[]string       `json:"tagIds"`
	UpdatedAt          *time.Time      `json:"updatedAt"` // For last-write-wins conflict detection
}

type GoalHandler struct {
	db *gorm.DB
}

func NewGoalHandler(db *gorm.DB) *GoalHandler {
	return &GoalHandler{db: db}
}

// getLastGoalInstance retrieves the most recently created goal instance for a given goal.
// Uses optimized query with composite index (goal_id, created_at) for fast lookup.
// Only loads minimal fields: ID, CreatedAt, PeriodStart, PeriodEnd, Status.
func (h *GoalHandler) getLastGoalInstance(goalID string) (*db.GoalInstance, error) {
	var instance db.GoalInstance

	err := h.db.
		Select("id", "created_at", "period_start", "period_end", "status").
		Where("goal_id = ?", goalID).
		Order("created_at DESC").
		Limit(1).
		First(&instance).
		Error

	if err == gorm.ErrRecordNotFound {
		return nil, nil // No instance found, not an error
	}

	if err != nil {
		return nil, err
	}

	return &instance, nil
}
