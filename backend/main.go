package main

// @title        Aether API
// @version      1.0
// @description  something, something to write in
// @BasePath     /v1

import (
	"log"

	"aether/internal/api"
	"aether/internal/db"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"
	fiberSwagger "github.com/swaggo/fiber-swagger"
)

func main() {

	// load env config
	godotenv.Load()

	database, err := db.Initialize()
	if err != nil {
		log.Fatal(err)
	}

	if err := db.Migrate(database); err != nil {
		log.Fatal(err)
	}

	app := fiber.New()

	// Swagger UI
	app.Get("/swagger/*", fiberSwagger.WrapHandler)

	api.RegisterRoutes(app, database)

	log.Println("Aether backend running on :9119")
	log.Fatal(app.Listen("0.0.0.0:9119"))
}
