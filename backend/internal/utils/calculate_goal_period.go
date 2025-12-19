package utils

import (
	"time"
)

type RecurringGoal struct {
	RecurrenceType     string
	RecurrenceInterval int
	RecurrenceAnchor   time.Time
}

func CalculateGoalPeriod(goal RecurringGoal, now time.Time) (time.Time, time.Time) {
	switch goal.RecurrenceType {

	case "daily":
		start := startOfDay(now)
		end := start.Add(24*time.Hour - time.Nanosecond)
		return start, end

	case "weekly":
		return calculateWeekly(goal, now)

	case "monthly":
		return calculateMonthly(goal, now)

	case "custom":
		return calculateCustom(goal, now)

	default:
		start := startOfDay(now)
		end := start.Add(24*time.Hour - time.Nanosecond)
		return start, end
	}
}

func startOfDay(t time.Time) time.Time {
	y, m, d := t.Date()
	return time.Date(y, m, d, 0, 0, 0, 0, t.Location())
}

func calculateWeekly(goal RecurringGoal, now time.Time) (time.Time, time.Time) {
	anchor := startOfDay(goal.RecurrenceAnchor)
	now = startOfDay(now)

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

func calculateMonthly(goal RecurringGoal, now time.Time) (time.Time, time.Time) {
	anchor := startOfDay(goal.RecurrenceAnchor)

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

func calculateCustom(goal RecurringGoal, now time.Time) (time.Time, time.Time) {
	anchor := startOfDay(goal.RecurrenceAnchor)
	now = startOfDay(now)

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
