package api

import (
	entryHandlers "aether/internal/handlers/entry"
	tagHandlers "aether/internal/handlers/tag"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func RegisterRoutes(app *fiber.App, gormDB *gorm.DB) {
	entryHandler := entryHandlers.NewEntryHandler(gormDB)
	tagHandler := tagHandlers.NewTagsHandler(gormDB)

	api := app.Group("/v1")

	api.Get("/ping", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"message": "pong"})
	})

	entryGroup := api.Group("/entry")

	entryGroup.Get("/", entryHandler.GetEntries)
	entryGroup.Get("/:id", entryHandler.GetEntryByID)
	entryGroup.Post("/", entryHandler.CreateEntry)
	entryGroup.Put("/:id", entryHandler.UpdateEntry)
	entryGroup.Delete("/:id", entryHandler.DeleteEntry)
	entryGroup.Post("/:id/tags", entryHandler.AddTagsToEntry)

	tagGroup := api.Group("/tags")
	tagGroup.Get("/", tagHandler.GetAllTags)
	tagGroup.Post("/", tagHandler.CreateTag)
}
