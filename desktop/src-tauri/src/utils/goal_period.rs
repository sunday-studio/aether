use crate::error::{AppError, Result};
use crate::utils::timezone::{days_since_in_timezone, get_goal_location, now_in_timezone};
use chrono::{DateTime, Datelike, Duration, Utc};
use chrono_tz::Tz;

#[derive(Clone)]
pub struct RecurringGoal {
    pub recurrence_type: String,
    pub recurrence_interval: i32,
    pub recurrence_anchor: DateTime<Utc>,
}

/// Calculate the period start and end times for a recurring goal.
/// All calculations are performed in the specified timezone (DST-aware).
pub fn calculate_goal_period(
    goal: RecurringGoal,
    now: DateTime<Utc>,
    tz: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    // Convert times to target timezone
    let anchor_in_tz = goal.recurrence_anchor.with_timezone(&tz);
    let now_in_tz = now.with_timezone(&tz);

    match goal.recurrence_type.as_str() {
        "daily" => calculate_daily(now_in_tz, tz),
        "weekly" => calculate_weekly(goal, anchor_in_tz, now_in_tz, tz),
        "monthly" => calculate_monthly(goal, anchor_in_tz, now_in_tz, tz),
        "yearly" => calculate_yearly(goal, anchor_in_tz, now_in_tz, tz),
        "custom" => calculate_custom(goal, anchor_in_tz, now_in_tz, tz),
        _ => {
            // Default to daily
            calculate_daily(now_in_tz, tz)
        }
    }
}

fn calculate_daily(now: DateTime<Tz>, tz: Tz) -> (DateTime<Utc>, DateTime<Utc>) {
    let t_in_tz = now.with_timezone(&tz);
    let start = t_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();
    let end = start + Duration::days(1) - Duration::nanoseconds(1);
    (start.with_timezone(&Utc), end.with_timezone(&Utc))
}

fn calculate_weekly(
    goal: RecurringGoal,
    anchor: DateTime<Tz>,
    now: DateTime<Tz>,
    tz: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let anchor_in_tz = anchor.with_timezone(&tz);
    let now_in_tz = now.with_timezone(&tz);
    let anchor_start = anchor_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();
    let now_start = now_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();

    let days_since = (now_start - anchor_start).num_days();
    let weeks_since = days_since / 7;

    let interval = if goal.recurrence_interval <= 0 {
        1
    } else {
        goal.recurrence_interval
    };

    let period_index = weeks_since / (interval as i64 * 7);

    let start = anchor_start + Duration::days(period_index * interval as i64 * 7);
    let end = start + Duration::days(interval as i64 * 7) - Duration::nanoseconds(1);

    (start.with_timezone(&Utc), end.with_timezone(&Utc))
}

fn calculate_monthly(
    goal: RecurringGoal,
    anchor: DateTime<Tz>,
    now: DateTime<Tz>,
    tz: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let anchor_in_tz = anchor.with_timezone(&tz);
    let now_in_tz = now.with_timezone(&tz);
    let anchor_start = anchor_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();
    let now_start = now_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();

    let interval = if goal.recurrence_interval <= 0 {
        1
    } else {
        goal.recurrence_interval
    };

    let months_since = (now_start.year() - anchor_start.year()) * 12
        + (now_start.month() as i32 - anchor_start.month() as i32);

    let period_index = months_since / interval;

    let start = anchor_start
        .with_month(anchor_start.month())
        .unwrap()
        .with_year(anchor_start.year())
        .unwrap();

    // Add months using chrono's checked_add_months equivalent
    let mut start = start;
    for _ in 0..(period_index * interval) {
        start = start.with_month(start.month() + 1).unwrap_or_else(|| {
            // If month overflow, go to next year
            start
                .with_year(start.year() + 1)
                .unwrap()
                .with_month(1)
                .unwrap()
        });
    }

    let mut end = start;
    for _ in 0..interval {
        end = end.with_month(end.month() + 1).unwrap_or_else(|| {
            end.with_year(end.year() + 1)
                .unwrap()
                .with_month(1)
                .unwrap()
        });
    }
    let end = end - Duration::nanoseconds(1);

    (start.with_timezone(&Utc), end.with_timezone(&Utc))
}

fn calculate_yearly(
    goal: RecurringGoal,
    anchor: DateTime<Tz>,
    now: DateTime<Tz>,
    tz: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let anchor_in_tz = anchor.with_timezone(&tz);
    let now_in_tz = now.with_timezone(&tz);
    let anchor_start = anchor_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();
    let now_start = now_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();

    let interval = if goal.recurrence_interval <= 0 {
        1
    } else {
        goal.recurrence_interval
    };

    let years_since = now_start.year() - anchor_start.year();
    let period_index = years_since / interval;

    let start = anchor_start
        .with_year(anchor_start.year() + period_index * interval)
        .unwrap();

    let end = start.with_year(start.year() + interval).unwrap() - Duration::nanoseconds(1);

    (start.with_timezone(&Utc), end.with_timezone(&Utc))
}

fn calculate_custom(
    goal: RecurringGoal,
    anchor: DateTime<Tz>,
    now: DateTime<Tz>,
    tz: Tz,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let anchor_in_tz = anchor.with_timezone(&tz);
    let now_in_tz = now.with_timezone(&tz);
    let anchor_start = anchor_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();
    let now_start = now_in_tz
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap();

    let interval = if goal.recurrence_interval <= 0 {
        1
    } else {
        goal.recurrence_interval
    };

    let days_since = (now_start - anchor_start).num_days();
    let period_index = days_since / (interval as i64);

    let start = anchor_start + Duration::days(period_index * interval as i64);
    let end = start + Duration::days(interval as i64) - Duration::nanoseconds(1);

    (start.with_timezone(&Utc), end.with_timezone(&Utc))
}

/// Check if a new goal instance should be created based on the goal's recurrence settings
/// and the last instance's creation date. All intervals are in days, regardless of recurrence type.
pub fn should_create_new_goal_instance(
    is_non_recurring: bool,
    last_instance_created_at: Option<DateTime<Utc>>,
    recurrence_interval: Option<i32>,
    timezone: Option<&str>,
) -> Result<bool> {
    // Non-recurring goals never create new instances
    if is_non_recurring {
        return Ok(false);
    }

    // If no last instance exists, we should create the first one
    if last_instance_created_at.is_none() {
        return Ok(true);
    }

    let last_instance = last_instance_created_at.unwrap();
    let tz_str = timezone.unwrap_or("UTC");
    let tz = get_goal_location(tz_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid timezone: {}", e)))?;
    let now = now_in_timezone(tz);

    // Calculate calendar days since last instance (DST-safe)
    let days_since = days_since_in_timezone(last_instance, now, tz);

    // Get the interval (all intervals are in days)
    let interval = recurrence_interval.unwrap_or(1);
    let interval = if interval <= 0 { 1 } else { interval };

    // For all recurrence types, check if days since last instance >= interval
    Ok(days_since >= interval as i64)
}
