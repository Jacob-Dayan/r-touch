// R-touch Library
// Copyright (c) 2026 Jacob Dayan
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 or the MIT License,
// at your option. You may not use this file except in compliance with
// one of these licenses.

use chrono::{Datelike, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Weekday};
use std::fmt;
use std::time::SystemTime;

/// Error type returned by [`parse_time_expression`] when an input string
/// cannot be converted to a [`std::time::SystemTime`].
///
/// Each variant carries a human-readable description of the problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeParseError {
    /// The string matched a known pattern but was structurally malformed
    /// (e.g. wrong number of digits, missing separators).
    InvalidFormat(String),
    /// The string was structurally valid but contained an out-of-range value
    /// (e.g. month 13, hour 25).
    InvalidValue(String),
    /// The string did not match any supported format or keyword expression.
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

pub fn parse_time_expression(input: &str) -> Result<SystemTime, TimeParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TimeParseError::InvalidFormat(
            "Time expression cannot be empty".to_string(),
        ));
    }

    let lower = trimmed.to_ascii_lowercase();
    let now = Local::now();

    if let Ok(st) = parse_standard_formats(trimmed) {
        return Ok(st);
    }

    if let Ok(dt) = parse_exact_keywords(&lower, &now) {
        return system_time_from_local_dt(dt);
    }

    if let Ok(dt) = parse_relative_offset(&lower, &now) {
        return system_time_from_local_dt(dt);
    }

    if let Ok(dt) = parse_relative_next_last(&lower, &now) {
        return system_time_from_local_dt(dt);
    }

    if let Some(ndt) = parse_relative_with_time(&lower, &now)? {
        return system_time_from_naive(ndt);
    }

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

fn parse_exact_keywords(
    lower: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    match lower {
        "now" | "today" => Ok(*now),
        "yesterday" => now
            .checked_sub_signed(chrono::Duration::days(1))
            .ok_or_else(|| TimeParseError::InvalidValue("Date underflow".to_string())),
        "tomorrow" => now
            .checked_add_signed(chrono::Duration::days(1))
            .ok_or_else(|| TimeParseError::InvalidValue("Date overflow".to_string())),
        _ => Err(TimeParseError::UnsupportedExpression("".to_string())),
    }
}

fn parse_relative_offset(
    lower: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    let parts: Vec<&str> = lower.split_whitespace().collect();

    if parts.len() == 3 {
        if parts[0] == "+" {
            let num: i64 = parts[1]
                .parse()
                .map_err(|_| TimeParseError::InvalidFormat("Invalid number".to_string()))?;
            return apply_offset(now, num, parts[2]);
        }
        if parts[0] == "-" {
            let num: i64 = parts[1]
                .parse()
                .map_err(|_| TimeParseError::InvalidFormat("Invalid number".to_string()))?;
            return apply_offset(now, -num, parts[2]);
        }
        if parts[2] == "ago" {
            let num: i64 = parts[0]
                .parse()
                .map_err(|_| TimeParseError::InvalidFormat("Invalid number".to_string()))?;
            return apply_offset(now, -num, parts[1]);
        }
    }

    if parts.len() == 2 {
        if let Some(num_str) = parts[0].strip_prefix('+') {
            let num: i64 = num_str
                .parse()
                .map_err(|_| TimeParseError::InvalidFormat("Invalid number".to_string()))?;
            return apply_offset(now, num, parts[1]);
        }
        if let Some(num_str) = parts[0].strip_prefix('-') {
            let num: i64 = num_str
                .parse()
                .map_err(|_| TimeParseError::InvalidFormat("Invalid number".to_string()))?;
            return apply_offset(now, -num, parts[1]);
        }
    }

    Err(TimeParseError::UnsupportedExpression("".to_string()))
}

fn apply_offset(
    now: &chrono::DateTime<Local>,
    val: i64,
    unit: &str,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    let dur = match unit {
        "second" | "seconds" | "sec" | "s" => chrono::Duration::seconds(val),
        "minute" | "minutes" | "min" | "m" => chrono::Duration::minutes(val),
        "hour" | "hours" | "hr" | "h" => chrono::Duration::hours(val),
        "day" | "days" | "d" => chrono::Duration::days(val),
        "week" | "weeks" | "w" => chrono::Duration::days(val * 7),
        "month" | "months" => {
            if val >= 0 {
                return now
                    .checked_add_months(Months::new(val as u32))
                    .ok_or_else(|| TimeParseError::InvalidValue("Month overflow".to_string()));
            } else {
                return now
                    .checked_sub_months(Months::new((-val) as u32))
                    .ok_or_else(|| TimeParseError::InvalidValue("Month underflow".to_string()));
            }
        }
        "year" | "years" | "y" => {
            let target_year = now.year() as i64 + val;
            return now
                .with_year(target_year as i32)
                .ok_or_else(|| TimeParseError::InvalidValue("Year overflow".to_string()));
        }
        _ => return Err(TimeParseError::UnsupportedExpression(unit.to_string())),
    };
    now.checked_add_signed(dur)
        .ok_or_else(|| TimeParseError::InvalidValue("Date offset overflow".to_string()))
}

fn parse_relative_next_last(
    lower: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    if let Some(rest) = lower.strip_prefix("next ") {
        return parse_relative_next(rest.trim(), now);
    }
    if let Some(rest) = lower.strip_prefix("last ") {
        return parse_relative_last(rest.trim(), now);
    }
    Err(TimeParseError::UnsupportedExpression("".to_string()))
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
        "Unknown target: '{target}'"
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
        "Unknown target: '{target}'"
    )))
}

fn parse_relative_date_base(
    lower: &str,
    now: &chrono::DateTime<Local>,
) -> Result<chrono::DateTime<Local>, TimeParseError> {
    if let Ok(dt) = parse_exact_keywords(lower, now) {
        return Ok(dt);
    }
    if let Ok(dt) = parse_relative_offset(lower, now) {
        return Ok(dt);
    }
    if let Ok(dt) = parse_relative_next_last(lower, now) {
        return Ok(dt);
    }
    Err(TimeParseError::UnsupportedExpression("".to_string()))
}

fn parse_relative_with_time(
    lower: &str,
    now: &chrono::DateTime<Local>,
) -> Result<Option<NaiveDateTime>, TimeParseError> {
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.len() < 2 {
        return Ok(None);
    }

    let time_str = tokens[tokens.len() - 1];
    if !time_str.contains(':') {
        return Ok(None);
    }

    let date_tokens = if tokens.len() >= 3 && tokens[tokens.len() - 2] == "at" {
        &tokens[..tokens.len() - 2]
    } else {
        &tokens[..tokens.len() - 1]
    };

    if date_tokens.is_empty() {
        return Ok(None);
    }

    let date_part = date_tokens.join(" ");

    if let Ok(dt) = parse_relative_date_base(&date_part, now) {
        let naive_time = parse_time_of_day(time_str)?;
        let ndt = NaiveDateTime::new(dt.date_naive(), naive_time);
        return Ok(Some(ndt));
    }

    if let Ok(nd) = NaiveDate::parse_from_str(&date_part, "%Y-%m-%d") {
        let naive_time = parse_time_of_day(time_str)?;
        let ndt = NaiveDateTime::new(nd, naive_time);
        return Ok(Some(ndt));
    }

    Ok(None)
}

fn parse_time_of_day(time_str: &str) -> Result<NaiveTime, TimeParseError> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(TimeParseError::InvalidFormat(format!(
            "Expected HH:MM or HH:MM:SS, got '{time_str}'"
        )));
    }

    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid hour: {}", parts[0])))?;
    if hour > 23 {
        return Err(TimeParseError::InvalidValue(format!("Invalid hour: {hour}")));
    }

    let min = parts[1]
        .parse::<u32>()
        .map_err(|_| TimeParseError::InvalidValue(format!("Invalid minute: {}", parts[1])))?;
    if min > 59 {
        return Err(TimeParseError::InvalidValue(format!("Invalid minute: {min}")));
    }

    let sec = if parts.len() == 3 {
        let s = parts[2]
            .parse::<u32>()
            .map_err(|_| TimeParseError::InvalidValue(format!("Invalid second: {}", parts[2])))?;
        if s > 59 {
            return Err(TimeParseError::InvalidValue(format!("Invalid second: {s}")));
        }
        s
    } else {
        0
    };

    NaiveTime::from_hms_opt(hour, min, sec).ok_or_else(|| {
        TimeParseError::InvalidValue(format!("Invalid time values: {hour:02}:{min:02}:{sec:02}"))
    })
}

fn parse_standard_formats(input: &str) -> Result<SystemTime, TimeParseError> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(input) {
        return Ok(SystemTime::from(dt));
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(input) {
        return Ok(SystemTime::from(dt));
    }

    let formats = [
        "%Y-%m-%d %H:%M:%S%.f %z",
        "%Y-%m-%d %H:%M:%S %z",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M",
    ];

    for fmt in formats {
        if let Ok(dt) = chrono::DateTime::parse_from_str(input, fmt) {
            return Ok(SystemTime::from(dt));
        }
        if let Ok(ndt) = NaiveDateTime::parse_from_str(input, fmt) {
            return system_time_from_naive(ndt);
        }
    }

    if let Ok(nd) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        if let Some(nt) = NaiveTime::from_hms_opt(0, 0, 0) {
            return system_time_from_naive(NaiveDateTime::new(nd, nt));
        }
    }

    Err(TimeParseError::UnsupportedExpression("".to_string()))
}

fn parse_gnu_touch_time(input: &str, current_year: i32) -> Result<NaiveDateTime, TimeParseError> {
    let (main_part, sec_part) = match input.split_once('.') {
        Some((main, sec)) => (main, Some(sec)),
        None => (input, None),
    };

    if !main_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(TimeParseError::InvalidFormat(format!(
            "Format requires digits: {main_part}"
        )));
    }

    let seconds = match sec_part {
        Some(sec) => {
            if sec.len() != 2 || !sec.chars().all(|c| c.is_ascii_digit()) {
                return Err(TimeParseError::InvalidFormat(format!(
                    "Seconds must be 2 digits: {sec}"
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
                "Expected 8, 10, or 12 digits, got {len}"
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
        assert!(parse_time_expression("2026-08-14 14:30:00+03:00").is_ok());
        assert!(parse_time_expression("2026-08-14T14:30:00Z").is_ok());
        assert!(parse_time_expression("Fri, 14 Aug 2026 14:30:00 +0000").is_ok());
    }

    #[test]
    fn test_parse_relative_expressions() {
        assert!(parse_time_expression("now").is_ok());
        assert!(parse_time_expression("today").is_ok());
        assert!(parse_time_expression("yesterday").is_ok());
        assert!(parse_time_expression("tomorrow").is_ok());

        assert!(parse_time_expression("2 days ago").is_ok());
        assert!(parse_time_expression("1 week ago").is_ok());
        assert!(parse_time_expression("+3 hours").is_ok());
        assert!(parse_time_expression("-15 minutes").is_ok());

        assert!(parse_time_expression("next tuesday").is_ok());
        assert!(parse_time_expression("next mon").is_ok());
        assert!(parse_time_expression("next month").is_ok());
        assert!(parse_time_expression("next year").is_ok());
        assert!(parse_time_expression("next january").is_ok());
        assert!(parse_time_expression("next dec").is_ok());

        assert!(parse_time_expression("last month").is_ok());
        assert!(parse_time_expression("today 14:30").is_ok());
        assert!(parse_time_expression("today at 14:30").is_ok());

        // Relative dates with time (yesterday HH:MM, tomorrow HH:MM, 1 week ago HH:MM, etc.)
        assert!(parse_time_expression("yesterday 14:30").is_ok());
        assert!(parse_time_expression("yesterday at 14:30:45").is_ok());
        assert!(parse_time_expression("tomorrow 09:15").is_ok());
        assert!(parse_time_expression("tomorrow at 09:15:00").is_ok());
        assert!(parse_time_expression("1 week ago 10:00").is_ok());
        assert!(parse_time_expression("1 week ago at 10:00").is_ok());
        assert!(parse_time_expression("2 days ago 16:20").is_ok());
        assert!(parse_time_expression("3 weeks ago 08:00").is_ok());
        assert!(parse_time_expression("+2 days 12:00").is_ok());
        assert!(parse_time_expression("-1 day 18:30").is_ok());
        assert!(parse_time_expression("next tuesday 15:45").is_ok());
        assert!(parse_time_expression("last friday at 20:00").is_ok());
    }

    #[test]
    fn test_parse_relative_with_time_values() {
        use chrono::Timelike;

        let now = Local::now();
        let st = parse_time_expression("yesterday 14:30").unwrap();
        let dt: chrono::DateTime<Local> = st.into();
        let expected_date = (now - chrono::Duration::days(1)).date_naive();
        assert_eq!(dt.date_naive(), expected_date);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 0);

        let st = parse_time_expression("tomorrow 08:15:20").unwrap();
        let dt: chrono::DateTime<Local> = st.into();
        let expected_date = (now + chrono::Duration::days(1)).date_naive();
        assert_eq!(dt.date_naive(), expected_date);
        assert_eq!(dt.hour(), 8);
        assert_eq!(dt.minute(), 15);
        assert_eq!(dt.second(), 20);

        let st = parse_time_expression("1 week ago 11:45").unwrap();
        let dt: chrono::DateTime<Local> = st.into();
        let expected_date = (now - chrono::Duration::days(7)).date_naive();
        assert_eq!(dt.date_naive(), expected_date);
        assert_eq!(dt.hour(), 11);
        assert_eq!(dt.minute(), 45);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_parse_invalid_expressions() {
        assert!(parse_time_expression("").is_err());
        assert!(parse_time_expression("   ").is_err());
        assert!(parse_time_expression("invalid_time_str").is_err());
        assert!(parse_time_expression("today 25:00").is_err());
        assert!(parse_time_expression("today 14:60").is_err());
        assert!(parse_time_expression("yesterday 25:00").is_err());
        assert!(parse_time_expression("yesterday 14:60").is_err());
        assert!(parse_time_expression("tomorrow 24:00").is_err());
        assert!(parse_time_expression("1 week ago 12:61").is_err());
        assert!(parse_time_expression("1 week ago 12:30:75").is_err());
        assert!(parse_time_expression("next invalidday").is_err());
        assert!(parse_time_expression("202608141430.123").is_err());
    }
}
