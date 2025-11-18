package entry

import (
	"github.com/gofiber/fiber/v2"
	"gorm.io/gorm"
)

func RegisterEntryRoutes(api *fiber.Group, gormDB *gorm.DB) {
	api.Post("/entries", func(c *fiber.Ctx) error { return CreateEntry(c, gormDB) })
	api.Get("/entries", func(c *fiber.Ctx) error { return GetEntries(c, gormDB) })
	api.Get("/entries/:id", func(c *fiber.Ctx) error { return GetEntryByID(c, gormDB) })
	api.Put("/entries/:id", func(c *fiber.Ctx) error { return UpdateEntry(c, gormDB) })
	api.Delete("/entries/:id", func(c *fiber.Ctx) error { return DeleteEntry(c, gormDB) })
}
