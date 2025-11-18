package db

import (
	"time"

	"gorm.io/gorm"
)

type Entry struct {
	ID         string         `json:"id" gorm:"primaryKey"`
	CreatedAt  time.Time      `json:"createdAt"`
	UpdatedAt  time.Time      `json:"updatedAt"`
	DeletedAt  gorm.DeletedAt `json:"deletedAt" gorm:"index"`
	Document   string         `json:"document" gorm:"type:text;not null"`
	IsPinned   bool           `json:"isPinned" gorm:"default:false"`
	IsArchived bool           `json:"isArchived" gorm:"default:false"`
	IsDeleted  bool           `json:"isDeleted" gorm:"default:false"`
	Tags       *[]Tag         `json:"tags" gorm:"many2many:entry_tags;"`
}

type Tag struct {
	ID        string         `json:"id" gorm:"primaryKey"`
	CreatedAt time.Time      `json:"createdAt"`
	UpdatedAt time.Time      `json:"updatedAt"`
	DeletedAt gorm.DeletedAt `json:"deletedAt" gorm:"index"`
	Name      string         `json:"name" gorm:"not null"`
}
