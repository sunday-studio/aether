package handlers

import (
	"time"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type CreateGoalPayload struct {
	Name               string         `json:"name"`
	Description        *string        `json:"description"`
	RecurrenceType     string         `json:"recurrenceType"`
	RecurrenceInterval int            `json:"recurrenceInterval"`
	RecurrenceAnchor   time.Time      `json:"recurrenceAnchor"`
	RecurrenceMeta     datatypes.JSON `json:"recurrenceMeta"`
	TagIDs             []string       `json:"tagIds"`
}

type UpdateGoalPayload struct {
	Name               *string         `json:"name"`
	Description        *string         `json:"description"`
	RecurrenceType     *string         `json:"recurrenceType"`
	RecurrenceInterval *int            `json:"recurrenceInterval"`
	RecurrenceAnchor   *time.Time      `json:"recurrenceAnchor"`
	RecurrenceMeta     *datatypes.JSON `json:"recurrenceMeta"`
	TagIDs             *[]string       `json:"tagIds"`
}

type GoalHandler struct {
	db *gorm.DB
}

func NewGoalHandler(db *gorm.DB) *GoalHandler {
	return &GoalHandler{db: db}
}
