package db

import (
	"aether/internal/utils"
	"time"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type Entry struct {
	ID        string    `json:"id" gorm:"primaryKey"`
	Document  string    `json:"document" gorm:"type:text;not null"`
	Tags      []Tag     `json:"tags" gorm:"many2many:entry_tags;"`
	CreatedAt time.Time `json:"createdAt"`

	IsPinned   bool `json:"isPinned" gorm:"default:false"`
	IsArchived bool `json:"isArchived" gorm:"default:false"`
	IsDeleted  bool `json:"isDeleted" gorm:"default:false"`

	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
}

type Tag struct {
	ID   string `json:"id" gorm:"primaryKey"`
	Name string `json:"name" gorm:"not null"`

	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
}

type Task struct {
	ID string `json:"id" gorm:"primaryKey"`

	Title       string  `json:"title" gorm:"not null"`
	Description *string `json:"description"`
	Tags        []Tag   `json:"tags" gorm:"many2many:task_tags;"`

	IsCompleted bool       `json:"isCompleted" gorm:"default:false"`
	DueDate     *time.Time `json:"dueDate" gorm:"index"`

	GoalInstanceID *string       `json:"goalInstanceId" gorm:"index"`
	GoalInstance   *GoalInstance `json:"goalInstance" swaggerignore:"true"`

	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
}

type Goal struct {
	ID          string  `json:"id" gorm:"primaryKey"`
	Name        string  `json:"name" gorm:"not null"`
	Description *string `json:"description"`

	RecurrenceType     string         `json:"recurrenceType" gorm:"not null"`     // daily | weekly | monthly | custom
	RecurrenceInterval int            `json:"recurrenceInterval" gorm:"not null"` // 1, 2, 25, etc
	RecurrenceAnchor   time.Time      `json:"recurrenceAnchor" gorm:"not null"`
	RecurrenceMeta     datatypes.JSON `json:"recurrenceMeta" swaggerignore:"true"`

	Tags []Tag `json:"tags" gorm:"many2many:goal_tags;"`

	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
}

type GoalInstance struct {
	ID string `gorm:"primaryKey"`

	GoalID string `gorm:"index;uniqueIndex:idx_goal_period"`
	Goal   Goal   `gorm:"constraint:OnDelete:CASCADE"`

	PeriodStart time.Time `json:"periodStart" gorm:"index;uniqueIndex:idx_goal_period"`
	PeriodEnd   time.Time `json:"periodEnd" gorm:"index"`

	Status string `json:"status" gorm:"not null"` // active | completed | skipped

	CreatedAt time.Time `json:"createdAt"`

	Tags  []Tag `json:"tags" gorm:"many2many:goal_instance_tags"`
	Tasks []Task
}

func (g *Goal) BeforeCreate(tx *gorm.DB) error {
	if g.ID == "" {
		g.ID = utils.GenerateID("goal")
	}
	return nil
}

func (gi *GoalInstance) BeforeCreate(tx *gorm.DB) error {
	if gi.ID == "" {
		gi.ID = utils.GenerateID("goal-instance")
	}
	return nil
}

func (t *Task) BeforeCreate(tx *gorm.DB) error {
	if t.ID == "" {
		t.ID = utils.GenerateID("task")
	}
	return nil
}

func (t *Tag) BeforeCreate(tx *gorm.DB) error {
	if t.ID == "" {
		t.ID = utils.GenerateID("tag")
	}
	return nil
}
