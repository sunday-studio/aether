package handlers

import (
	"aether/internal/db"
	"aether/internal/logging"

	"github.com/gofiber/fiber/v2"
)

type SyncHandler struct {
	log *logging.Logger
}

func NewSyncHandler() *SyncHandler {
	return &SyncHandler{
		log: logging.NewLogger(),
	}
}

// Sync godoc
// @Id sync
// @Summary Sync local replica with remote database
// @Description Manually triggers a sync between the local replica and the remote Turso database
// @Tags Sync
// @Produce json
// @Success 200 {object} map[string]interface{}
// @Failure 500 {object} map[string]string
// @Router /sync [post]
func (h *SyncHandler) Sync(c *fiber.Ctx) error {
	connector := db.GetConnector()
	if connector == nil {
		return c.Status(500).JSON(fiber.Map{
			"error":   "sync_not_available",
			"message": "Replica mode is not enabled or connector is not available",
		})
	}

	h.log.Info("Manual sync triggered")
	syncResult, err := connector.Sync()
	if err != nil {
		h.log.Error("Manual sync failed", "error", err)
		return c.Status(500).JSON(fiber.Map{
			"error":   "sync_failed",
			"message": err.Error(),
		})
	}

	h.log.Info("Manual sync completed", "framesSynced", syncResult.FramesSynced)
	return c.JSON(fiber.Map{
		"success":      true,
		"framesSynced": syncResult.FramesSynced,
		"message":      "Sync completed successfully",
	})
}
