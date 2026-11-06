use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

// Value of a datetime-local field ("2026-10-28T18:02"), read in the offset recorded with the session.
// chrono's `FromStr` / `Display` are used rather than `format` / `parse_from_str`, whose
// format-string machinery would only be compiled into the .wasm for these fields.
pub fn time_input_value(time: DateTime<FixedOffset>) -> String {
    let local = time.naive_local();
    format!("{}T{:02}:{:02}", local.date(), local.hour(), local.minute())
}

pub fn parse_time_input(value: &str, offset: FixedOffset) -> Option<DateTime<FixedOffset>> {
    let (date, time) = value.trim().split_once('T')?;
    let date: NaiveDate = date.parse().ok()?;
    let time: NaiveTime = time.parse().ok()?;
    NaiveDateTime::new(date, time)
        .and_local_timezone(offset)
        .single()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::at;

    #[test]
    fn round_trips_in_the_session_offset() {
        let time = at(28, 18, 2);
        assert_eq!(time_input_value(time), "2026-10-28T18:02");
        assert_eq!(
            parse_time_input("2026-10-28T18:02", *time.offset()),
            Some(time)
        );
        assert_eq!(parse_time_input("", *time.offset()), None);
        assert_eq!(parse_time_input("2026-10-28 18:02", *time.offset()), None);
        assert_eq!(parse_time_input("2026-10-28T25:02", *time.offset()), None);
    }

    #[test]
    fn pads_single_digit_hours_and_minutes() {
        assert_eq!(time_input_value(at(3, 7, 5)), "2026-10-03T07:05");
    }
}
