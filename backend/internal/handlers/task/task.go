package handlers

import (
	"time"

	"gorm.io/gorm"
)

type CreateTaskPayload struct {
	Title          string    `json:"title"`
	Description    *string   `json:"description"`
	DueDate        time.Time `json:"dueDate"`
	GoalInstanceID *string   `json:"goalInstanceId"`
	TagIDs         []string  `json:"tagIds"`
}

type UpdateTaskPayload struct {
	Title          *string    `json:"title"`
	Description    *string    `json:"description"`
	DueDate        *time.Time `json:"dueDate"`
	IsCompleted    *bool      `json:"isCompleted"`
	GoalInstanceID *string    `json:"goalInstanceId"`
	TagIDs         *[]string  `json:"tagIds"`
}

type TaskHandler struct {
	db *gorm.DB
}

func NewTaskHandler(db *gorm.DB) *TaskHandler {
	return &TaskHandler{db: db}
}
