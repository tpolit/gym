use domain::Db;

use crate::shared::decimal::format_scaled;
use crate::shared::format::format_short_date;

const HISTORY_LIMIT: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyWeightView {
    pub input: String,
    pub history: Vec<BodyWeightRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyWeightRow {
    pub date_text: String,
    pub weight_text: String,
}

// Most recent first; the input is prefilled with the latest weight.
pub fn body_weight_view(db: &Db) -> BodyWeightView {
    let log = db.preferences().body_weights();
    BodyWeightView {
        input: log
            .latest()
            .map(|(_, weight)| format_scaled(weight.grams(), 3))
            .unwrap_or_default(),
        history: log
            .entries()
            .rev()
            .take(HISTORY_LIMIT)
            .map(|(date, weight)| BodyWeightRow {
                date_text: format_short_date(date),
                weight_text: format!("{} kg", format_scaled(weight.grams(), 3)),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::set_body_weight;
    use crate::test_support::date;

    #[test]
    fn lists_weights_most_recent_first() {
        let mut db = Db::default();
        assert_eq!(body_weight_view(&db).input, "");
        set_body_weight(&mut db, "80", date(10)).unwrap();
        set_body_weight(&mut db, "78,5", date(28)).unwrap();
        let view = body_weight_view(&db);
        assert_eq!(view.input, "78,5");
        assert_eq!(
            view.history[0],
            BodyWeightRow {
                date_text: "mer. 28 oct.".to_owned(),
                weight_text: "78,5 kg".to_owned(),
            }
        );
        assert_eq!(view.history[1].weight_text, "80 kg");
    }
}
