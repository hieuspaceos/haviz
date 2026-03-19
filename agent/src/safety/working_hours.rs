use chrono::{Datelike, Local, Timelike, Weekday};

use crate::safety::SafetyResult;

// Allowed operating hours (24h format, inclusive start, exclusive end)
const WEEKDAY_HOURS: (u8, u8) = (7, 22);   // Mon–Fri: 07:00–22:00
const WEEKEND_HOURS: (u8, u8) = (8, 20);   // Sat–Sun: 08:00–20:00
const LUNCH_BREAK: (u8, u8) = (12, 13);    // 12:00–13:00 — reduce speed 50%

/// Returns true when the current local time falls on a weekend.
fn is_weekend() -> bool {
    matches!(Local::now().weekday(), Weekday::Sat | Weekday::Sun)
}

/// Returns true when the current local hour is the lunch break window.
pub fn is_lunch_hour() -> bool {
    let hour = Local::now().hour() as u8;
    hour >= LUNCH_BREAK.0 && hour < LUNCH_BREAK.1
}

/// Calculate the next datetime when sending is permitted.
/// Moves to the next morning's open hour (same day if still before start, else next day).
pub fn next_available_time() -> chrono::DateTime<Local> {
    let now = Local::now();
    let today = now.date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);

    // Determine whether tomorrow is a weekday or weekend
    let tomorrow_weekday = chrono::NaiveDate::from_ymd_opt(
        tomorrow.year(),
        tomorrow.month(),
        tomorrow.day(),
    )
    .map(|d| d.weekday());

    let open_hour = match tomorrow_weekday {
        Some(Weekday::Sat) | Some(Weekday::Sun) => WEEKEND_HOURS.0,
        _ => WEEKDAY_HOURS.0,
    };

    // Build next-morning datetime at the open hour
    chrono::NaiveDateTime::new(
        tomorrow,
        chrono::NaiveTime::from_hms_opt(open_hour as u32, 0, 0).unwrap(),
    )
    .and_local_timezone(Local)
    .single()
    .unwrap_or_else(|| now + chrono::Duration::hours(12))
}

/// Layer 3 safety check: ensures the agent only operates during acceptable hours.
///
/// Returns:
/// - `Allow`          — within working hours (caller should halve speed if lunch)
/// - `Queue{send_at}` — outside working hours; schedule for next open window
pub fn check() -> SafetyResult {
    let now = Local::now();
    let hour = now.hour() as u8;

    let (start, end) = if is_weekend() {
        WEEKEND_HOURS
    } else {
        WEEKDAY_HOURS
    };

    if hour < start || hour >= end {
        return SafetyResult::Queue {
            reason: format!(
                "Outside working hours ({start}:00–{end}:00). Message queued for next window."
            ),
            send_at: next_available_time(),
        };
    }

    // Within hours — lunch note is advisory only (caller reduces rate)
    SafetyResult::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_available_time_is_in_future() {
        let next = next_available_time();
        assert!(next > Local::now());
    }

    #[test]
    fn test_is_lunch_hour_returns_bool() {
        // Just verify it compiles and returns without panic
        let _ = is_lunch_hour();
    }

    #[test]
    fn test_check_returns_valid_variant() {
        // Result must be Allow or Queue — never Block for working-hours check
        let result = check();
        match result {
            SafetyResult::Allow | SafetyResult::Queue { .. } => {}
            SafetyResult::Block { .. } => panic!("working_hours::check must not return Block"),
        }
    }
}
