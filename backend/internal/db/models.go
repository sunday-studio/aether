package db

import (
	"aether/internal/utils"
	"time"

	"gorm.io/datatypes"
	"gorm.io/gorm"
)

type SchemaMigration struct {
	ID        uint      `gorm:"primaryKey"`
	Version   string    `gorm:"unique;not null"` // e.g., "20260107_fix_corrupted_ids"
	Name      string    `gorm:"not null"`
	AppliedAt time.Time `gorm:"not null"`
}

type Settings struct {
	ID        string    `json:"id" gorm:"primaryKey"`
	Timezone  string    `json:"timezone" gorm:"not null;default:'UTC'"` // IANA timezone name
	CreatedAt time.Time `json:"createdAt" gorm:"autoCreateTime"`
	UpdatedAt time.Time `json:"updatedAt" gorm:"autoUpdateTime;default:(datetime('now'))"`
}

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

	GoalInstanceID *string `json:"goalInstanceId" gorm:"index"`
	GoalID         *string `json:"goalId" gorm:"index"`

	SubTasks []SubTask `json:"subTasks" gorm:"foreignKey:TaskID;constraint:OnDelete:CASCADE"`

	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
}

type SubTask struct {
	ID          string         `json:"id" gorm:"primaryKey"`
	Title       string         `json:"title"`
	IsCompleted bool           `json:"isCompleted" gorm:"default:false"`
	TaskID      string         `json:"taskId" gorm:"index"`
	CreatedAt   time.Time      `json:"createdAt"`
	UpdatedAt   time.Time      `json:"updatedAt"`
	DeletedAt   gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
	OrderIndex  int            `json:"orderIndex" gorm:"not null"`
}

type Goal struct {
	ID          string  `json:"id" gorm:"primaryKey"`
	Name        string  `json:"name" gorm:"not null"`
	Description *string `json:"description"`

	RecurrenceType     string         `json:"recurrenceType" gorm:"not null"`     // bi-weekly | weekly | monthly | custom
	RecurrenceInterval int            `json:"recurrenceInterval" gorm:"not null"` // 1, 2, 25, etc
	RecurrenceAnchor   time.Time      `json:"recurrenceAnchor" gorm:"not null"`
	RecurrenceMeta     datatypes.JSON `json:"recurrenceMeta" swaggerignore:"true"`
	Timezone           string         `json:"timezone" gorm:"not null;default:'UTC'"` // IANA timezone name, snapshot at creation

	Tags []Tag `json:"tags" gorm:"many2many:goal_tags;"`

	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"-" gorm:"index" swaggerignore:"true"`
}

type GoalInstance struct {
	ID string `json:"id" gorm:"primaryKey"`

	GoalID string `json:"goalId" gorm:"index;index:idx_goal_created_at;uniqueIndex:idx_goal_period"`
	Goal   Goal   `json:"goal" gorm:"constraint:OnDelete:CASCADE"`

	PeriodStart time.Time `json:"periodStart" gorm:"index;uniqueIndex:idx_goal_period"`
	PeriodEnd   time.Time `json:"periodEnd" gorm:"index"`

	Status string `json:"status" gorm:"not null"` // active | completed | skipped

	CreatedAt time.Time `json:"createdAt" gorm:"index:idx_goal_created_at"`

	Tags  []Tag  `json:"tags" gorm:"many2many:goal_instance_tags"`
	Tasks []Task `json:"tasks"`
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

func (s *Settings) BeforeCreate(tx *gorm.DB) error {
	if s.ID == "" {
		s.ID = utils.GenerateID("settings")
	}
	if s.Timezone == "" {
		s.Timezone = "UTC"
	}
	return nil
}

func (st *SubTask) BeforeCreate(tx *gorm.DB) error {
	if st.ID == "" {
		st.ID = utils.GenerateID("subtask")
	}
	return nil
}
