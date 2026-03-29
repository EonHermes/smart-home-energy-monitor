use chrono::{DateTime, Duration, Utc};

/// Helper functions for time-related operations

/// Get the start of the current day (midnight)
pub fn start_of_day(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.with_hour(0).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap()
}

/// Get the end of the current day (23:59:59.999999999)
pub fn end_of_day(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.with_hour(23).unwrap()
        .with_minute(59).unwrap()
        .with_second(59).unwrap()
        .with_nanosecond(999_999_999).unwrap()
}

/// Get the start of the current week (Monday)
pub fn start_of_week(dt: DateTime<Utc>) -> DateTime<Utc> {
    let days_since_monday = dt.weekday().num_days_from_monday() as i64;
    dt - Duration::days(days_since_monday)
}

/// Get the start of the current month
pub fn start_of_month(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt.with_day(1).unwrap()
        .with_hour(0).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap()
}

/// Format duration in human-readable format
pub fn format_duration(hours: f64) -> String {
    if hours < 1.0 {
        format!("{} minutes", (hours * 60.0) as u32)
    } else if hours < 24.0 {
        format!("{:.1} hours", hours)
    } else {
        let days = (hours / 24.0) as u32;
        let remaining_hours = (hours % 24.0) as u32;
        if remaining_hours > 0 {
            format!("{} days {} hours", days, remaining_hours)
        } else {
            format!("{} days", days)
        }
    }
}

/// Get time range labels for a given period
pub fn get_period_label(hours: u32) -> &'static str {
    match hours {
        1 => "Last Hour",
        6 => "Last 6 Hours",
        24 => "Last 24 Hours",
        48 => "Last 48 Hours",
        72 => "Last 3 Days",
        168 => "Last Week",
        720 => "Last Month",
        _ => "Custom Period",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_of_day() {
        let dt = Utc::now();
        let start = start_of_day(dt);
        
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.5), "30 minutes");
        assert_eq!(format_duration(2.5), "2.5 hours");
        assert_eq!(format_duration(25.0), "1 days 1 hours");
        assert_eq!(format_duration(48.0), "2 days");
    }

    #[test]
    fn test_period_label() {
        assert_eq!(get_period_label(24), "Last 24 Hours");
        assert_eq!(get_period_label(168), "Last Week");
        assert_eq!(get_period_label(500), "Custom Period");
    }
}
