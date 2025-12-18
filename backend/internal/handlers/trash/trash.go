package handlers

import (
	"gorm.io/gorm"
)

type TrashHandler struct {
	db *gorm.DB
}

func NewTrashHandler(db *gorm.DB) *TrashHandler {
	return &TrashHandler{db: db}
}
