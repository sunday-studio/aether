package utils

import (
	"time"
)

type RecurringGoal struct {
	RecurrenceType     string
	RecurrenceInterval int
	RecurrenceAnchor   time.Time
}

// CalculateGoalPeriod calculates the period start and end times for a recurring goal.
// If loc is nil, uses the timezone from the provided times.
// If loc is provided, all calculations are performed in that timezone (DST-aware).
func CalculateGoalPeriod(goal RecurringGoal, now time.Time, loc *time.Location) (time.Time, time.Time) {
	// If no timezone provided, use the timezone from the anchor or now
	if loc == nil {
		loc = goal.RecurrenceAnchor.Location()
		if loc == time.UTC && now.Location() != time.UTC {
			loc = now.Location()
		}
	}

	// Convert times to target timezone
	anchorInTZ := goal.RecurrenceAnchor.In(loc)
	nowInTZ := now.In(loc)

	switch goal.RecurrenceType {

	case "daily":
		start := startOfDayInTimezone(nowInTZ, loc)
		end := start.Add(24*time.Hour - time.Nanosecond)
		return start, end

	case "weekly":
		return calculateWeekly(goal, anchorInTZ, nowInTZ, loc)

	case "monthly":
		return calculateMonthly(goal, anchorInTZ, nowInTZ, loc)

	case "yearly":
		return calculateYearly(goal, anchorInTZ, nowInTZ, loc)

	case "custom":
		return calculateCustom(goal, anchorInTZ, nowInTZ, loc)

	default:
		start := startOfDayInTimezone(nowInTZ, loc)
		end := start.Add(24*time.Hour - time.Nanosecond)
		return start, end
	}
}

// startOfDayInTimezone returns the start of day in the specified timezone.
// This is a helper that uses the timezone utility function.
func startOfDayInTimezone(t time.Time, loc *time.Location) time.Time {
	return StartOfDayInTimezone(t, loc)
}

func calculateWeekly(goal RecurringGoal, anchor, now time.Time, loc *time.Location) (time.Time, time.Time) {
	anchor = startOfDayInTimezone(anchor, loc)
	now = startOfDayInTimezone(now, loc)

	daysSince := int(now.Sub(anchor).Hours() / 24)
	weeksSince := daysSince / 7

	interval := goal.RecurrenceInterval
	if interval <= 0 {
		interval = 1
	}

	periodIndex := weeksSince / interval

	start := anchor.AddDate(0, 0, periodIndex*interval*7)
	end := start.AddDate(0, 0, interval*7).Add(-time.Nanosecond)

	return start, end
}

func calculateMonthly(goal RecurringGoal, anchor, now time.Time, loc *time.Location) (time.Time, time.Time) {
	anchor = startOfDayInTimezone(anchor, loc)
	now = startOfDayInTimezone(now, loc)

	interval := goal.RecurrenceInterval
	if interval <= 0 {
		interval = 1
	}

	monthsSince :=
		(now.Year()-anchor.Year())*12 +
			int(now.Month()) -
			int(anchor.Month())

	periodIndex := monthsSince / interval

	start := anchor.AddDate(0, periodIndex*interval, 0)
	end := start.AddDate(0, interval, 0).Add(-time.Nanosecond)

	return start, end
}

func calculateYearly(goal RecurringGoal, anchor, now time.Time, loc *time.Location) (time.Time, time.Time) {
	anchor = startOfDayInTimezone(anchor, loc)
	now = startOfDayInTimezone(now, loc)

	interval := goal.RecurrenceInterval
	if interval <= 0 {
		interval = 1
	}

	yearsSince := now.Year() - anchor.Year()
	periodIndex := yearsSince / interval

	// AddDate handles leap years automatically
	start := anchor.AddDate(periodIndex*interval, 0, 0)
	end := start.AddDate(interval, 0, 0).Add(-time.Nanosecond)

	return start, end
}

func calculateCustom(goal RecurringGoal, anchor, now time.Time, loc *time.Location) (time.Time, time.Time) {
	anchor = startOfDayInTimezone(anchor, loc)
	now = startOfDayInTimezone(now, loc)

	interval := goal.RecurrenceInterval
	if interval <= 0 {
		interval = 1
	}

	daysSince := int(now.Sub(anchor).Hours() / 24)
	periodIndex := daysSince / interval

	start := anchor.AddDate(0, 0, periodIndex*interval)
	end := start.AddDate(0, 0, interval).Add(-time.Nanosecond)

	return start, end
}
