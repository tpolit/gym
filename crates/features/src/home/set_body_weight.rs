use chrono::NaiveDate;
use domain::{BodyWeight, Db};

use crate::shared::decimal::parse_scaled;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyWeightInputError {
    NotANumber,
    OutOfRange,
}

pub const fn body_weight_error_message(error: BodyWeightInputError) -> &'static str {
    match error {
        BodyWeightInputError::NotANumber => "Saisissez un poids, par exemple 72,5.",
        BodyWeightInputError::OutOfRange => "Le poids doit être compris entre 20 et 300 kg.",
    }
}

// "72,5" kg, recorded for today; a second entry the same day replaces it.
pub fn set_body_weight(
    db: &mut Db,
    typed: &str,
    today: NaiveDate,
) -> Result<(), BodyWeightInputError> {
    let grams = parse_scaled(typed, 3).ok_or(BodyWeightInputError::NotANumber)?;
    let weight = BodyWeight::from_grams(grams).map_err(|_| BodyWeightInputError::OutOfRange)?;
    db.record_body_weight(today, weight);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::date;

    #[test]
    fn records_the_typed_weight_for_today() {
        let mut db = Db::default();
        set_body_weight(&mut db, "72,5", date(30)).unwrap();
        let latest = db.preferences().body_weights().latest();
        assert_eq!(
            latest.map(|(day, weight)| (day, weight.grams())),
            Some((date(30), 72_500))
        );
    }

    #[test]
    fn rejects_invalid_or_implausible_weights() {
        let mut db = Db::default();
        assert_eq!(
            set_body_weight(&mut db, "abc", date(30)),
            Err(BodyWeightInputError::NotANumber)
        );
        assert_eq!(
            set_body_weight(&mut db, "7", date(30)),
            Err(BodyWeightInputError::OutOfRange)
        );
        assert_eq!(db.preferences().body_weights().latest(), None);
    }
}
