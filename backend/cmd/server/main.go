package main

import (
	"log"

	"aether/internal/api"
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
)

func main() {
	database, err := db.Initialize()
	if err != nil {
		log.Fatal(err)
	}

	if err := db.Migrate(database); err != nil {
		log.Fatal(err)
	}

	app := fiber.New()

	api.RegisterRoutes(app, database)

	log.Println("Aether backend running on :9119")
	log.Fatal(app.Listen(":9119"))
}
