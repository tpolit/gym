use chrono::NaiveDate;

use crate::shared::format::format_short_date;

pub fn default_session_name(today: NaiveDate) -> String {
    format!("Séance du {}", format_short_date(today))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_session_after_the_day() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        assert_eq!(default_session_name(today), "Séance du lun. 2 nov.");
    }
}
