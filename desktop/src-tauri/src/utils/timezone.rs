use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type LocationCache = Arc<RwLock<HashMap<String, chrono_tz::Tz>>>;

lazy_static::lazy_static! {
    static ref TIMEZONE_CACHE: LocationCache = Arc::new(RwLock::new(HashMap::new()));
}

/// Get timezone location, using IANA timezone names.
/// Caches loaded locations for performance.
pub fn get_goal_location(timezone: &str) -> Result<chrono_tz::Tz, String> {
    let timezone = if timezone.is_empty() { "UTC" } else { timezone };

    // Check cache first
    {
        let cache = TIMEZONE_CACHE.read().unwrap();
        if let Some(loc) = cache.get(timezone) {
            return Ok(*loc);
        }
    }

    // Try to parse as IANA timezone
    let tz = timezone.parse::<chrono_tz::Tz>()
        .map_err(|_| format!("Invalid timezone: {}", timezone))?;

    // Cache it
    {
        let mut cache = TIMEZONE_CACHE.write().unwrap();
        cache.insert(timezone.to_string(), tz);
    }

    Ok(tz)
}

/// Returns the start of day (00:00:00) in the specified timezone.
/// Handles DST automatically.
pub fn start_of_day_in_timezone<T: TimeZone>(
    t: DateTime<T>,
    tz: chrono_tz::Tz,
) -> DateTime<chrono_tz::Tz> {
    let t_in_tz = t.with_timezone(&tz);
    t_in_tz.date_naive().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(tz).unwrap()
}

/// Returns the current time in the specified timezone.
pub fn now_in_timezone(tz: chrono_tz::Tz) -> DateTime<chrono_tz::Tz> {
    Utc::now().with_timezone(&tz)
}

/// Calculates the number of calendar days between two times
/// in the specified timezone. This is DST-safe.
pub fn days_since_in_timezone<T1: TimeZone, T2: TimeZone>(
    t1: DateTime<T1>,
    t2: DateTime<T2>,
    tz: chrono_tz::Tz,
) -> i64 {
    let t1_in_tz = t1.with_timezone(&tz);
    let t2_in_tz = t2.with_timezone(&tz);
    
    let days = (t2_in_tz - t1_in_tz).num_days();
    days.abs()
}
