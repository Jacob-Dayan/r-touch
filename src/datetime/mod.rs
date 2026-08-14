use chrono::{Datelike, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Weekday};
use std::fmt;
use std::time::SystemTime;

/// Error type for time expression parsing failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeParseError {
    InvalidFormat(String),
    InvalidValue(String),
    UnsupportedExpression(String),
}

impl fmt::Display for TimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeParseError::InvalidFormat(msg) => write!(f, "Invalid time format: {msg}"),
            TimeParseError::InvalidValue(msg) => write!(f, "Invalid date/time value: {msg}"),
            TimeParseError::UnsupportedExpression(msg) => {
                write!(f, "Unsupported time expression: {msg}")
            }
        }
    }
}

impl std::error::Error for TimeParseError {}

/// Parse a time expression (relative, absolute, or GNU touch format) into a [`SystemTime`].
pub fn parse_time_expression(input: &str) -> Result<SystemTime, TimeParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TimeParseError::InvalidFormat(
            "Time expression cannot be empty".to_string(),
        ));
    }

    let lower = trimmed.to_ascii_lowercase();
    let now = Local::now();

    // Check for relative expressions: "next ...", "last ...", "today ..."
    if let Some(rest) = lower.strip_prefix("next ") {
        let dt = parse_relative_next(rest.trim(), &now)?;
        return system_time_from_local_dt(dt);
    }

    if let Some(rest) = lower.strip_prefix("last ") {
        let dt = parse_relative_last(rest.trim(), &now)?;
        return system_time_from_local_dt(dt);
    }

    if let Some(rest) = lower.strip_prefix("today ") {
        let dt = parse_today_time(rest.trim(), &now)?;
        return system_time_from_local_dt(dt);
    }

    // Try ISO format parsing (e.g., YYYY-MM-DD HH:MM:SS)
    if let Some(ndt) = parse_iso_time(trimmed) {
        return system_time_from_naive(ndt);
    }

    // Try GNU touch format parsing ([[CC]YY]MMDDhhmm[.ss])
    if let Ok(ndt) = parse_gnu_touch_time(trimmed, now.year()) {
        return system_time_from_naive(ndt);
    }

    Err(TimeParseError::UnsupportedExpression(format!(
        "Unable to parse time expression '{input}'"
    )))
}

fn system_time_from_local_dt(dt: chrono::DateTime<Local>) -> Result<SystemTime, TimeParseError> {
    Ok(SystemTime::from(dt))
}

fn system_time_from_naive(ndt: NaiveDateTime) -> Result<SystemTime, TimeParseError> {
    Local
        .from_local_datetime(&ndt)
        .earliest()
        .map(SystemTime::from)
        .ok_or_else(|| {
            TimeParseError::InvalidValue(format!(
                "Date/time cannot be converted to local time: {ndt}"
            ))
        })
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

fn parse_month(s: &str) -> Option<u32> {
    match s {
        "january" | "jan" => Some(1),
        "february" | "feb" => Some(2),
        "march" | "mar" => Some(3),
        "april" | "apr" => Some(4),
        "may" => Some(5),
        "june" | "jun" => Some(6),
        "july" | "jul" => Some(7),
        "august" | "aug" => Some(8),
        "september" | "sep" => Some(9),
        "october" | "oct" => Some(10),
        "november" | "nov" => Some(11),
        "december" | "dec" => Some(12),
        _ => None,
    }
}

fn parse_relative_next(
    target: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    if let Some(weekday) = parse_weekday(target) {
        let cur_num = now.weekday().num_days_from_monday();
        let target_num = weekday.num_days_from_monday();
        let days_ahead = if target_num > cur_num {
            target_num - cur_num
        } else {
            7 - (cur_num - target_num)
        };
        let days_ahead = if days_ahead == 0 { 7 } else { days_ahead };
        return now
            .checked_add_signed(chrono::Duration::days(days_ahead as i64))
            .ok_or_else(|| TimeParseError::InvalidValue("Date overflow".to_string()));
    }

    if target == "month" {
        return now
            .checked_add_months(Months::new(1))
            .ok_or_else(|| TimeParseError::InvalidValue("Month overflow".to_string()));
    }

    if target == "year" {
        return now
            .with_year(now.year() + 1)
            .ok_or_else(|| TimeParseError::InvalidValue("Year overflow".to_string()));
    }

    if let Some(target_m) = parse_month(target) {
        let cur_m = now.month();
        let year_add = if target_m > cur_m { 0 } else { 1 };
        let target_year = now.year() + year_add;
        return now
            .with_year(target_year)
            .and_then(|dt| dt.with_month(target_m))
            .ok_or_else(|| {
                TimeParseError::InvalidValue(format!(
                    "Invalid date for target month: {target_year}-{target_m}"
                ))
            });
    }

    Err(TimeParseError::UnsupportedExpression(format!(
        "Unknown 'next' expression target: '{target}'"
    )))
}

fn parse_relative_last(
    target: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    if let Some(weekday) = parse_weekday(target) {
        let cur_num = now.weekday().num_days_from_monday();
        let target_num = weekday.num_days_from_monday();
        let days_back = if cur_num > target_num {
            cur_num - target_num
        } else {
            7 - (target_num - cur_num)
        };
        let days_back = if days_back == 0 { 7 } else { days_back };
        return now
            .checked_sub_signed(chrono::Duration::days(days_back as i64))
            .ok_or_else(|| TimeParseError::InvalidValue("Date underflow".to_string()));
    }

    if target == "month" {
        return now
            .checked_sub_months(Months::new(1))
            .ok_or_else(|| TimeParseError::InvalidValue("Month underflow".to_string()));
    }

    if target == "year" {
        return now
            .with_year(now.year() - 1)
            .ok_or_else(|| TimeParseError::InvalidValue("Year underflow".to_string()));
    }

    if let Some(target_m) = parse_month(target) {
        let cur_m = now.month();
        let year_sub = if target_m < cur_m { 0 } else { 1 };
        let target_year = now.year() - year_sub;
        return now
            .with_year(target_year)
            .and_then(|dt| dt.with_month(target_m))
            .ok_or_else(|| {
                TimeParseError::InvalidValue(format!(
                    "Invalid date for target month: {target_year}-{target_m}"
                ))
            });
    }

    Err(TimeParseError::UnsupportedExpression(format!(
        "Unknown 'last' expression target: '{target}'"
    )))
}

fn parse_today_time(
    time_str: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(TimeParseError::InvalidFormat(format!(
            "Expected HH:MM or HH:MM:SS for 'today', got '{time_str}'"
        )));
    }

    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid hour: {}", parts[0])))?;
    let min = parts[1]
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid minute: {}", parts[1])))?;
    let sec = if parts.len() == 3 {
        parts[2]
            .parse::<u32>()
            .map_err(|_| TimeParseError::InvalidValue(format!("Invalid second: {}", parts[2])))?
    } else {
        0
    };

    let naive_time = NaiveTime::from_hms_opt(hour, min, sec).ok_or_else(|| {
        TimeParseError::InvalidValue(format!("Invalid time values: {hour:02}:{min:02}:{sec:02}"))
    })?;

    let naive_date = now.date_naive();
    let ndt = NaiveDateTime::new(naive_date, naive_time);

    Local
        .from_local_datetime(&ndt)
        .earliest()
        .ok_or_else(|| TimeParseError::InvalidValue("Local time conversion error".to_string()))
}

fn parse_iso_time(input: &str) -> Option<NaiveDateTime> {
    if let Ok(ndt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
        return Some(ndt);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        return Some(ndt);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M") {
        return Some(ndt);
    }
    if let Ok(nd) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        let nt = NaiveTime::from_hms_opt(0, 0, 0)?;
        return Some(NaiveDateTime::new(nd, nt));
    }
    None
}

fn parse_gnu_touch_time(input: &str, current_year: i32) -> Result<NaiveDateTime, TimeParseError> {
    let (main_part, sec_part) = match input.split_once('.') {
        Some((main, sec)) => (main, Some(sec)),
        None => (input, None),
    };

    if !main_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(TimeParseError::InvalidFormat(format!(
            "GNU touch format requires digits: {main_part}"
        )));
    }

    let seconds = match sec_part {
        Some(sec) => {
            if sec.len() != 2 || !sec.chars().all(|c| c.is_ascii_digit()) {
                return Err(TimeParseError::InvalidFormat(format!(
                    "GNU touch seconds must be 2 digits: {sec}"
                )));
            }
            sec.parse::<u32>()
                .map_err(|_| TimeParseError::InvalidValue(format!("Invalid seconds: {sec}")))?
        }
        None => 0,
    };

    let len = main_part.len();
    let (year, month_str, day_str, hour_str, min_str) = match len {
        8 => (
            current_year,
            &main_part[0..2],
            &main_part[2..4],
            &main_part[4..6],
            &main_part[6..8],
        ),
        10 => {
            let yy = main_part[0..2]
                .parse::<i32>()
                .map_err(|_| TimeParseError::InvalidValue("Invalid 2-digit year".to_string()))?;
            let full_year = if (0..=68).contains(&yy) {
                2000 + yy
            } else {
                1900 + yy
            };
            (
                full_year,
                &main_part[2..4],
                &main_part[4..6],
                &main_part[6..8],
                &main_part[8..10],
            )
        }
        12 => {
            let ccyy = main_part[0..4]
                .parse::<i32>()
                .map_err(|_| TimeParseError::InvalidValue("Invalid 4-digit year".to_string()))?;
            (
                ccyy,
                &main_part[4..6],
                &main_part[6..8],
                &main_part[8..10],
                &main_part[10..12],
            )
        }
        _ => {
            return Err(TimeParseError::InvalidFormat(format!(
                "GNU touch time expected 8, 10, or 12 digits, got {len}"
            )));
        }
    };

    let month = month_str
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid month: {month_str}")))?;
    let day = day_str
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid day: {day_str}")))?;
    let hour = hour_str
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid hour: {hour_str}")))?;
    let min = min_str
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid minute: {min_str}")))?;

    let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
        TimeParseError::InvalidValue(format!("Invalid date: {year}-{month:02}-{day:02}"))
    })?;
    let time = NaiveTime::from_hms_opt(hour, min, seconds).ok_or_else(|| {
        TimeParseError::InvalidValue(format!("Invalid time: {hour:02}:{min:02}:{seconds:02}"))
    })?;

    Ok(NaiveDateTime::new(date, time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gnu_touch_formats() {
        assert!(parse_time_expression("202608141430").is_ok());
        assert!(parse_time_expression("2608141430.45").is_ok());
        assert!(parse_time_expression("08141430").is_ok());
    }

    #[test]
    fn test_parse_iso_formats() {
        assert!(parse_time_expression("2026-08-14 14:30:00").is_ok());
        assert!(parse_time_expression("2026-08-14T14:30:00").is_ok());
        assert!(parse_time_expression("2026-08-14").is_ok());
    }

    #[test]
    fn test_parse_relative_expressions() {
        assert!(parse_time_expression("next tuesday").is_ok());
        assert!(parse_time_expression("next mon").is_ok());
        assert!(parse_time_expression("next month").is_ok());
        assert!(parse_time_expression("next year").is_ok());
        assert!(parse_time_expression("next january").is_ok());
        assert!(parse_time_expression("next dec").is_ok());

        assert!(parse_time_expression("last month").is_ok());
        assert!(parse_time_expression("last friday").is_ok());
        assert!(parse_time_expression("last sun").is_ok());
        assert!(parse_time_expression("last year").is_ok());
        assert!(parse_time_expression("last august").is_ok());

        assert!(parse_time_expression("today 14:30").is_ok());
        assert!(parse_time_expression("today 09:15:30").is_ok());
        assert!(parse_time_expression("today 00:00").is_ok());
    }

    #[test]
    fn test_parse_invalid_expressions() {
        assert!(parse_time_expression("").is_err());
        assert!(parse_time_expression("   ").is_err());
        assert!(parse_time_expression("invalid_time_str").is_err());
        assert!(parse_time_expression("today 25:00").is_err());
        assert!(parse_time_expression("today 14:60").is_err());
        assert!(parse_time_expression("next invalidday").is_err());
        assert!(parse_time_expression("202608141430.123").is_err());
    }
}
