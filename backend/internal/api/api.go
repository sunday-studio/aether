package api

import (
	entryHandler "aether/internal/handlers/entry"

	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func RegisterRoutes(app *fiber.App, gormDB *gorm.DB) {
	entryHandler := entryHandler.NewEntryHandler(gormDB)

	api := app.Group("/v1")

	api.Get("/ping", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"message": "pong"})
	})

}
