package db

import "gorm.io/gorm"

func SeedDatabase(db *gorm.DB) error {
	return nil
}

// import (
// 	"fmt"
// 	"math/rand"
// 	"time"

// 	"aether/internal/utils"

// 	"gorm.io/datatypes"
// 	"gorm.io/gorm"
// )

// func SeedDatabase(db *gorm.DB) error {
// 	fmt.Println("🌱 Seeding database...")

// 	rand.Seed(time.Now().UnixNano())

// 	// ---- Goals -------------------------------------------------

// 	goals := []Goal{
// 		{
// 			Name:               "Weekly Household Chores",
// 			Description:        ptr("Things to keep the house running"),
// 			RecurrenceType:     "weekly",
// 			RecurrenceInterval: 1,
// 			RecurrenceAnchor:   time.Now().AddDate(0, 0, -21),
// 			RecurrenceMeta:     emptyJSON(),
// 		},
// 		{
// 			Name:               "Daily Planning",
// 			Description:        ptr("Plan each day"),
// 			RecurrenceType:     "daily",
// 			RecurrenceInterval: 1,
// 			RecurrenceAnchor:   time.Now().AddDate(0, 0, -14),
// 			RecurrenceMeta:     emptyJSON(),
// 		},
// 		{
// 			Name:               "Gym Cycle",
// 			Description:        ptr("Workout every 3 days"),
// 			RecurrenceType:     "custom",
// 			RecurrenceInterval: 3,
// 			RecurrenceAnchor:   time.Now().AddDate(0, 0, -60),
// 			RecurrenceMeta:     emptyJSON(),
// 		},
// 		{
// 			Name:               "Monthly Review",
// 			Description:        ptr("Reflect and plan ahead"),
// 			RecurrenceType:     "monthly",
// 			RecurrenceInterval: 1,
// 			RecurrenceAnchor:   time.Now().AddDate(0, -6, 0),
// 			RecurrenceMeta:     emptyJSON(),
// 		},
// 	}

// 	for i := range goals {
// 		if err := db.Create(&goals[i]).Error; err != nil {
// 			return err
// 		}
// 	}

// 	// ---- GoalInstances + Tasks --------------------------------

// 	for _, goal := range goals {
// 		seedGoalHistory(db, goal, 6)
// 	}

// 	// ---- Standalone Tasks -------------------------------------

// 	standaloneTitles := []string{
// 		"Buy groceries",
// 		"Reply to emails",
// 		"Book dentist appointment",
// 		"Clean workspace",
// 		"Backup laptop",
// 		"Read 10 pages",
// 		"Water plants",
// 		"Pay electricity bill",
// 		"Fix bike light",
// 		"Organize photos",
// 	}

// 	for i := 0; i < 20; i++ {
// 		task := Task{
// 			Title:       standaloneTitles[rand.Intn(len(standaloneTitles))],
// 			Description: ptr("Standalone task"),
// 			DueDate:     time.Now().AddDate(0, 0, rand.Intn(14)-7),
// 			IsCompleted: rand.Intn(2) == 1,
// 		}

// 		if err := db.Create(&task).Error; err != nil {
// 			return err
// 		}
// 	}

// 	fmt.Println("✅ Seeding complete")
// 	return nil
// }

// // -------------------------------------------------------------
// // Helpers
// // -------------------------------------------------------------

// func seedGoalHistory(db *gorm.DB, goal Goal, periods int) {
// 	now := time.Now()

// 	for i := periods; i >= 0; i-- {
// 		pointInTime := subtractPeriod(goal, now, i)

// 		start, end := utils.CalculateGoalPeriod(
// 			utils.RecurringGoal{
// 				RecurrenceType:     goal.RecurrenceType,
// 				RecurrenceInterval: goal.RecurrenceInterval,
// 				RecurrenceAnchor:   goal.RecurrenceAnchor,
// 			},
// 			pointInTime,
// 		)

// 		var instance GoalInstance
// 		err := db.
// 			Where("goal_id = ? AND period_start = ?", goal.ID, start).
// 			First(&instance).
// 			Error

// 		if err == nil {
// 			continue
// 		}

// 		instance = GoalInstance{
// 			GoalID:      goal.ID,
// 			PeriodStart: start,
// 			PeriodEnd:   end,
// 			Status:      randomStatus(i),
// 		}

// 		if err := db.Create(&instance).Error; err != nil {
// 			panic(err)
// 		}

// 		seedTasksForInstance(db, instance)
// 	}
// }

// func seedTasksForInstance(db *gorm.DB, instance GoalInstance) {
// 	titles := []string{
// 		"Clean kitchen",
// 		"Vacuum floor",
// 		"Go for a run",
// 		"Stretching routine",
// 		"Meal prep",
// 		"Write journal entry",
// 		"Plan next week",
// 		"Review finances",
// 		"Organize notes",
// 		"Read a chapter",
// 	}

// 	taskCount := rand.Intn(5) + 3

// 	for i := 0; i < taskCount; i++ {
// 		task := Task{
// 			Title:          titles[rand.Intn(len(titles))],
// 			Description:    ptr("Generated task"),
// 			DueDate:        instance.PeriodStart.AddDate(0, 0, rand.Intn(5)),
// 			IsCompleted:    rand.Intn(3) == 0,
// 			GoalInstanceID: &instance.ID,
// 		}

// 		if err := db.Create(&task).Error; err != nil {
// 			panic(err)
// 		}
// 	}
// }

// func randomStatus(backlog int) string {
// 	if backlog == 0 {
// 		return "active"
// 	}
// 	if rand.Intn(3) == 0 {
// 		return "completed"
// 	}
// 	return "skipped"
// }

// func subtractPeriod(goal Goal, t time.Time, count int) time.Time {
// 	switch goal.RecurrenceType {
// 	case "daily":
// 		return t.AddDate(0, 0, -count)
// 	case "weekly":
// 		return t.AddDate(0, 0, -7*count)
// 	case "monthly":
// 		return t.AddDate(0, -count, 0)
// 	case "custom":
// 		return t.AddDate(0, 0, -goal.RecurrenceInterval*count)
// 	default:
// 		return t
// 	}
// }

// func ptr[T any](v T) *T {
// 	return &v
// }

// func emptyJSON() datatypes.JSON {
// 	return datatypes.JSON([]byte(`{}`))
// }
