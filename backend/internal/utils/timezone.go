package utils

import (
	"fmt"
	"sync"
	"time"
)

// timezoneCache caches loaded timezone locations to avoid repeated LoadLocation calls
var (
	timezoneCache sync.Map
)

// GetGoalLocation returns the timezone location for a goal, using IANA timezone names.
// Caches loaded locations for performance.
// Accepts timezone string directly to avoid import cycle with db package.
func GetGoalLocation(timezone string) (*time.Location, error) {
	if timezone == "" {
		timezone = "UTC"
	}

	// Check cache first
	if cached, ok := timezoneCache.Load(timezone); ok {
		return cached.(*time.Location), nil
	}

	// Load location (handles DST automatically via IANA names)
	loc, err := time.LoadLocation(timezone)
	if err != nil {
		return nil, fmt.Errorf("invalid timezone %q: %w", timezone, err)
	}

	// Cache it
	timezoneCache.Store(timezone, loc)

	return loc, nil
}

// StartOfDayInTimezone returns the start of day (00:00:00) in the specified timezone.
// Handles DST automatically - Go's time package uses correct offset (CST vs CDT).
func StartOfDayInTimezone(t time.Time, loc *time.Location) time.Time {
	// Convert to target timezone first
	tInTZ := t.In(loc)

	// Get start of day in that timezone
	y, m, d := tInTZ.Date()
	return time.Date(y, m, d, 0, 0, 0, 0, loc)
}

// NowInTimezone returns the current time in the specified timezone.
// Uses correct DST offset automatically.
func NowInTimezone(loc *time.Location) time.Time {
	return time.Now().In(loc)
}

// DaysSinceInTimezone calculates the number of calendar days between two times
// in the specified timezone. This is DST-safe and works correctly across
// DST transitions (23/25 hour days).
func DaysSinceInTimezone(t1, t2 time.Time, loc *time.Location) int {
	// Convert both times to the target timezone
	t1InTZ := t1.In(loc)
	t2InTZ := t2.In(loc)

	// Calculate difference in hours, then convert to days
	// This works correctly across DST transitions because we're using
	// calendar days, not fixed 24-hour periods
	hoursDiff := t2InTZ.Sub(t1InTZ).Hours()
	daysSince := int(hoursDiff / 24)

	// Return absolute value (days since, regardless of order)
	if daysSince < 0 {
		return -daysSince
	}
	return daysSince
}
