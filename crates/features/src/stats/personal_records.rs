use domain::records::personal_records as domain_personal_records;
use domain::{Db, ExerciseId};

use crate::shared::format::format_short_date;
use crate::shared::records::{format_record_value, record_label};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLine {
    pub label: &'static str,
    pub value_text: String,
    pub date_text: String,
}

pub fn personal_records(db: &Db, exercise_id: &ExerciseId) -> Vec<RecordLine> {
    domain_personal_records(db, exercise_id)
        .into_iter()
        .map(|record| RecordLine {
            label: record_label(record.kind),
            value_text: format_record_value(record.kind, record.value),
            date_text: format_short_date(record.achieved_at.date_naive()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn lists_records_with_their_date() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(85, 3)])]);
        let records = personal_records(&db, &bench());
        assert_eq!(
            records[0],
            RecordLine {
                label: "Poids max",
                value_text: "85 kg".to_owned(),
                date_text: "mar. 27 oct.".to_owned(),
            }
        );
        assert_eq!(records.len(), 3);
    }
}
