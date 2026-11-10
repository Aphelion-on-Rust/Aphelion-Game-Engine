use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |time| time.as_secs())
}

/// Today's date (UTC) like 2026-11-10.
pub fn today() -> String {
    iso_date(now_unix())
}

/// Seconds since 1970 -> "YYYY-MM-DD".
pub fn iso_date(unix_seconds: u64) -> String {
    let (year, month, day) = civil_from_days((unix_seconds / 86_400) as i64);

    format!("{year:04}-{month:02}-{day:02}")
}

/// Days since 1970-01-01 -> (year, month, day). Standard calendar arithmetic.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = (if shifted >= 0 { shifted } else { shifted - 146_096 }) / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = (if month_index < 10 { month_index + 3 } else { month_index - 9 }) as u32;
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates_come_out_right() {
        assert_eq!(iso_date(0), "1970-01-01");
        assert_eq!(iso_date(951_782_400), "2000-02-29"); // a leap day
        assert_eq!(iso_date(1_700_000_000), "2023-11-14");
    }
}
