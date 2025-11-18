package db

import (
	"time"

	"gorm.io/gorm"
)

type Entry struct {
	ID         uint           `json:"id" gorm:"primaryKey"`
	CreatedAt  time.Time      `json:"createdAt"`
	UpdatedAt  time.Time      `json:"updatedAt"`
	DeletedAt  gorm.DeletedAt `json:"deletedAt" gorm:"index"`
	Document   string         `json:"document" gorm:"type:text;not null"`
	IsPinned   bool           `json:"isPinned" gorm:"default:false"`
	IsArchived bool           `json:"isArchived" gorm:"default:false"`
	IsDeleted  bool           `json:"isDeleted" gorm:"default:false"`
}
