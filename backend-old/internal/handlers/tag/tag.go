package tag

import (
	"gorm.io/gorm"
)

type TagsHandler struct {
	db *gorm.DB
}

func NewTagsHandler(db *gorm.DB) *TagsHandler {
	return &TagsHandler{db: db}
}
