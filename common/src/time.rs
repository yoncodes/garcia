use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: i64 = 86_400;
const MONDAY_EPOCH_DAY: i32 = 4;

pub struct ServerTime;

impl ServerTime {
    pub fn now_seconds() -> i64 {
        since_epoch().as_secs() as i64
    }

    pub fn now_seconds_i32() -> i32 {
        Self::now_seconds() as i32
    }

    pub fn now_millis() -> u128 {
        since_epoch().as_millis()
    }

    pub fn now_nanos() -> u128 {
        since_epoch().as_nanos()
    }
}

fn since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

pub fn day_index(timestamp: i32) -> i32 {
    timestamp.div_euclid(SECONDS_PER_DAY as i32)
}

pub fn daily_refresh_day(timestamp: i32, zone_offset: i32, refresh_hour: i32) -> i32 {
    daily_refresh_day_i64(timestamp, zone_offset, refresh_hour) as i32
}

pub fn weekly_refresh_index(timestamp: i32, zone_offset: i32, refresh_hour: i32) -> i32 {
    (daily_refresh_day(timestamp, zone_offset, refresh_hour) - MONDAY_EPOCH_DAY).div_euclid(7)
}

fn daily_refresh_day_i64(timestamp: i32, zone_offset: i32, refresh_hour: i32) -> i64 {
    i64::from(timestamp)
        .saturating_add(i64::from(zone_offset))
        .saturating_sub(i64::from(refresh_hour).saturating_mul(3_600))
        .div_euclid(SECONDS_PER_DAY)
}

pub fn seconds_until(end: i64, now: i64) -> i32 {
    end.saturating_sub(now).clamp(0, i64::from(i32::MAX)) as i32
}

pub fn table_time_utc(value: &str, zone_offset: i32) -> Option<i64> {
    parse_table_time(value)?.checked_sub(i64::from(zone_offset))
}

pub fn table_window_remaining(open: &str, close: &str, now: i32, zone_offset: i32) -> Option<i32> {
    let start = table_time_utc(open, zone_offset)?;
    let end = table_time_utc(close, zone_offset)?;
    let now = i64::from(now);
    (start <= now && now < end).then(|| seconds_until(end, now))
}

pub fn table_window_active(open: &str, close: &str, now: i32, zone_offset: i32) -> bool {
    table_window_remaining(open, close, now, zone_offset).is_some()
}

pub fn next_daily_refresh_seconds(timestamp: i32, zone_offset: i32, refresh_hour: i32) -> i32 {
    let day = daily_refresh_day_i64(timestamp, zone_offset, refresh_hour);
    let next = (day + 1)
        .saturating_mul(SECONDS_PER_DAY)
        .saturating_add(i64::from(refresh_hour).saturating_mul(3_600))
        .saturating_sub(i64::from(zone_offset));
    seconds_until(next, i64::from(timestamp))
}

pub fn next_weekly_refresh_seconds(timestamp: i32, zone_offset: i32, refresh_hour: i32) -> i32 {
    let day = daily_refresh_day(timestamp, zone_offset, refresh_hour);
    let days_since_monday = (day - MONDAY_EPOCH_DAY).rem_euclid(7);
    let next_day = i64::from(day + (7 - days_since_monday));
    let next = next_day
        .saturating_mul(SECONDS_PER_DAY)
        .saturating_add(i64::from(refresh_hour).saturating_mul(3_600))
        .saturating_sub(i64::from(zone_offset));
    seconds_until(next, i64::from(timestamp))
}

pub fn next_month_refresh_seconds(timestamp: i32, zone_offset: i32, refresh_hour: i32) -> i32 {
    let shifted = i64::from(timestamp)
        .saturating_add(i64::from(zone_offset))
        .saturating_sub(i64::from(refresh_hour).saturating_mul(3_600));
    let (year, month, _) = civil_from_days(shifted.div_euclid(SECONDS_PER_DAY));
    let (year, month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let next = days_from_civil(year, month, 1)
        .saturating_mul(SECONDS_PER_DAY)
        .saturating_add(i64::from(refresh_hour).saturating_mul(3_600))
        .saturating_sub(i64::from(zone_offset));
    seconds_until(next, i64::from(timestamp))
}

fn parse_table_time(value: &str) -> Option<i64> {
    let parts: Vec<i64> = value
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let [year, month, day, hour, minute, second] = parts.as_slice() else {
        return None;
    };
    if !(1..=12).contains(month)
        || !(1..=31).contains(day)
        || !(0..=23).contains(hour)
        || !(0..=59).contains(minute)
        || !(0..=59).contains(second)
    {
        return None;
    }
    Some(
        days_from_civil(*year, *month, *day)
            .saturating_mul(SECONDS_PER_DAY)
            .saturating_add(hour.saturating_mul(3_600))
            .saturating_add(minute.saturating_mul(60))
            .saturating_add(*second),
    )
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_game_table_time_with_zone_offset() {
        assert_eq!(table_time_utc("1970-01-01 04:00:00", -14_400), Some(28_800));
        assert_eq!(
            table_window_remaining(
                "1970-01-01 04:00:00",
                "1970-01-01 05:00:00",
                30_000,
                -14_400,
            ),
            Some(2_400)
        );
        assert!(!table_window_active(
            "1970-01-01 04:00:00",
            "1970-01-01 05:00:00",
            32_400,
            -14_400,
        ));
    }

    #[test]
    fn computes_daily_and_monthly_refreshes() {
        let now = 1_787_512_077;
        assert_eq!(next_daily_refresh_seconds(now, -14_400, 4), 46_323);
        assert_eq!(next_weekly_refresh_seconds(now, -14_400, 4), 46_323);
        assert_eq!(next_month_refresh_seconds(now, -14_400, 4), 737_523);
    }

    #[test]
    fn computes_day_indexes_and_countdowns() {
        assert_eq!(day_index(-1), -1);
        assert_eq!(day_index(86_400), 1);
        assert_eq!(daily_refresh_day(86_399, 0, 0), 0);
        assert_eq!(seconds_until(20, 7), 13);
        assert_eq!(seconds_until(7, 20), 0);
    }
}
