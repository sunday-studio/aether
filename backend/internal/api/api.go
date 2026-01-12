package api

import (
	entryHandlers "aether/internal/handlers/entry"
	goalHandlers "aether/internal/handlers/goal"
	syncHandlers "aether/internal/handlers/sync"
	tagHandlers "aether/internal/handlers/tag"
	taskHandlers "aether/internal/handlers/task"
	trashHandlers "aether/internal/handlers/trash"

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
	taskHandler := taskHandlers.NewTaskHandler(gormDB)
	trashHandler := trashHandlers.NewTrashHandler(gormDB)
	goalHandler := goalHandlers.NewGoalHandler(gormDB)
	syncHandler := syncHandlers.NewSyncHandler()

	api := app.Group("/v1")

	api.Get("/ping", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{"message": "pong pong"})
	})

	// sync
	api.Post("/sync", syncHandler.Sync)

	// trash
	trashGroup := api.Group("/trash")
	trashGroup.Get("/tasks", trashHandler.GetTrashedTasks)
	trashGroup.Post("/tasks/:id/restore", trashHandler.RestoreTask)

	// entries
	entryGroup := api.Group("/entry")
	entryGroup.Get("/", entryHandler.GetEntries)
	entryGroup.Get("/:id", entryHandler.GetEntryByID)
	entryGroup.Post("/", entryHandler.CreateEntry)
	entryGroup.Post("/bulk-create", entryHandler.BulkCreateEntries)
	entryGroup.Put("/:id", entryHandler.UpdateEntry)
	entryGroup.Delete("/:id", entryHandler.DeleteEntry)
	entryGroup.Post("/:id/tags", entryHandler.AddTagsToEntry)
	entryGroup.Delete("/:id/tags", entryHandler.RemoveTagsFromEntry)

	// tags
	tagGroup := api.Group("/tags")
	tagGroup.Get("/", tagHandler.GetAllTags)
	tagGroup.Post("/", tagHandler.CreateTag)
	tagGroup.Post("/bulk-create", tagHandler.BulkCreateTags)

	// tasks
	taskGroup := api.Group("/tasks")
	taskGroup.Post("/", taskHandler.CreateTask)
	taskGroup.Get("/inbox", taskHandler.GetInboxTasks)
	taskGroup.Get("/overdue", taskHandler.GetOverdueTasks)
	taskGroup.Post("/:id/tags", taskHandler.AddTagsToTask)
	taskGroup.Delete("/:id/tags", taskHandler.RemoveTagsFromTask)
	taskGroup.Post("/:id/goal", taskHandler.AddGoalToTask)
	taskGroup.Delete("/:id/goal", taskHandler.RemoveGoalFromTask)
	taskGroup.Get("/:id", taskHandler.GetTaskByID)
	taskGroup.Put("/:id", taskHandler.UpdateTask)
	taskGroup.Delete("/:id", taskHandler.DeleteTask)

	// subtasks
	taskGroup.Get("/:taskId/subtasks", taskHandler.GetSubTasks)
	taskGroup.Post("/:taskId/subtasks", taskHandler.CreateSubTask)
	taskGroup.Put("/:taskId/subtasks/:subtaskId", taskHandler.UpdateSubTask)
	taskGroup.Delete("/:taskId/subtasks/:subtaskId", taskHandler.DeleteSubTask)
	taskGroup.Post("/:taskId/subtasks/reorder", taskHandler.ReorderSubTasks)

	// goals
	goalGroup := api.Group("/goals")
	goalGroup.Get("/", goalHandler.GetGoals)
	goalGroup.Get("/:id", goalHandler.GetGoalByID)
	goalGroup.Post("/", goalHandler.CreateGoal)
	goalGroup.Put("/:id", goalHandler.UpdateGoal)
	goalGroup.Delete("/:id", goalHandler.DeleteGoal)
	goalGroup.Get("/:goalId/instances", goalHandler.GetGoalInstances)
	goalGroup.Get("/:goalId/instances/current", goalHandler.GetCurrentGoalInstance)
}
