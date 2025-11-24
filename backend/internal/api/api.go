package api

import (
	entryHandlers "aether/internal/handlers/entry"
	tagHandlers "aether/internal/handlers/tag"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"gorm.io/gorm"
)

func RegisterRoutes(app *fiber.App, gormDB *gorm.DB) {
	// Add CORS middleware
	app.Use(cors.New(cors.Config{
		AllowOrigins: "*",
		AllowHeaders: "Origin, Content-Type, Accept, Authorization",
		AllowMethods: "GET,POST,PUT,DELETE,OPTIONS",
	}))

	entryHandler := entryHandlers.NewEntryHandler(gormDB)
	tagHandler := tagHandlers.NewTagsHandler(gormDB)

	api := app.Group("/v1")

	api.Get("/ping", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"message": "pong pong"})
	})

	entryGroup := api.Group("/entry")

	entryGroup.Get("/", entryHandler.GetEntries)
	entryGroup.Get("/:id", entryHandler.GetEntryByID)
	entryGroup.Post("/", entryHandler.CreateEntry)
	entryGroup.Post("/bulk-create", entryHandler.BulkCreateEntries)
	entryGroup.Put("/:id", entryHandler.UpdateEntry)
	entryGroup.Delete("/:id", entryHandler.DeleteEntry)
	entryGroup.Post("/:id/tags", entryHandler.AddTagsToEntry)

	tagGroup := api.Group("/tags")
	tagGroup.Get("/", tagHandler.GetAllTags)
	tagGroup.Post("/", tagHandler.CreateTag)
}
