use chrono::NaiveDate;

// Value exchanged with a date input field: "2026-11-03", empty when there is no date.
// chrono's `FromStr` / `Display` already use this ISO format (and keep its format-string
// machinery out of the .wasm).
pub fn parse_date_input(value: &str) -> Option<NaiveDate> {
    value.trim().parse().ok()
}

pub fn date_input_value(date: Option<NaiveDate>) -> String {
    date.map(|date| date.to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_dates_and_handles_empty_values() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 3).unwrap();
        assert_eq!(parse_date_input("2026-11-03"), Some(date));
        assert_eq!(date_input_value(Some(date)), "2026-11-03");
        assert_eq!(parse_date_input(""), None);
        assert_eq!(parse_date_input("03/11/2026"), None);
        assert_eq!(date_input_value(None), "");
    }
}
