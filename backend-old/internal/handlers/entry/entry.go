package handlers

import (
	"gorm.io/gorm"
)

type EntryHandler struct {
	db *gorm.DB
}

func NewEntryHandler(db *gorm.DB) *EntryHandler {
	return &EntryHandler{db: db}
}
