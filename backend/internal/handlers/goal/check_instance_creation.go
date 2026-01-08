package handlers

import (
	"aether/internal/db"
	"aether/internal/utils"
)

// checkCreateNewGoalInstance determines if a new goal instance should be created
// based on the goal's recurrence settings and the last instance's creation date.
// All intervals are in days, regardless of recurrence type.
// Uses goal's timezone for all date comparisons (DST-aware).
func (h *GoalHandler) checkCreateNewGoalInstance(goal db.Goal, lastInstance *db.GoalInstance) (bool, error) {
	// If no last instance exists, we should create the first one
	if lastInstance == nil {
		return true, nil
	}

	// Get goal's timezone location
	loc, err := utils.GetGoalLocation(goal.Timezone)
	if err != nil {
		return false, err
	}

	// Convert current time and last instance date to goal's timezone
	nowInTZ := utils.NowInTimezone(loc)
	lastInstanceDateInTZ := lastInstance.CreatedAt.In(loc)

	// Calculate calendar days since last instance (DST-safe)
	daysSince := utils.DaysSinceInTimezone(lastInstanceDateInTZ, nowInTZ, loc)

	// Get the interval (all intervals are in days)
	interval := goal.RecurrenceInterval
	if interval <= 0 {
		interval = 1
	}

	// For all recurrence types, check if days since last instance >= interval
	// This is a simple day-based check regardless of recurrence type
	if daysSince >= interval {
		return true, nil
	}

	return false, nil
}
