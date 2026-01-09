package handlers

import (
	"time"

	"gorm.io/gorm"
)

type CreateTaskPayload struct {
	Title       string     `json:"title"`
	Description *string    `json:"description"`
	DueDate     *time.Time `json:"dueDate"`
	GoalID      *string    `json:"goalId"`
	TagIDs      []string   `json:"tagIds"`
}

type UpdateTaskPayload struct {
	Title       *string    `json:"title"`
	Description *string    `json:"description"`
	DueDate     *time.Time `json:"dueDate"`
	IsCompleted *bool      `json:"isCompleted"`
	GoalID      *string    `json:"goalId"`
	TagIDs      *[]string  `json:"tagIds"`
	UpdatedAt   *time.Time `json:"updatedAt"` // For last-write-wins conflict detection
}

type CreateSubTaskPayload struct {
	Title string `json:"title"`
}

type UpdateSubTaskPayload struct {
	Title       *string `json:"title"`
	IsCompleted *bool   `json:"isCompleted"`
}

type ReorderSubTasksPayload struct {
	SubTaskIDs []string `json:"subTaskIds"` // Ordered list of subtask IDs
}

type TaskHandler struct {
	db *gorm.DB
}

func NewTaskHandler(db *gorm.DB) *TaskHandler {
	return &TaskHandler{db: db}
}
