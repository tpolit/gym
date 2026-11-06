use domain::{Db, EntryId, SessionId, SetMeasure, WorkoutSet};

use super::last_performance::LastPerformance;
use crate::shared::format::format_set_measure;
use crate::shared::set_draft::SetDraft;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRows {
    pub done: Vec<DoneSetRow>,
    pub current_number: usize,
    pub current_previous: Option<String>,
    pub upcoming: Vec<UpcomingSetRow>,
    pub next_draft: SetDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoneSetRow {
    pub index: usize,
    pub number: usize,
    pub text: String,
    pub measure: SetMeasure,
    pub warmup: bool,
    pub previous: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpcomingSetRow {
    pub number: usize,
    pub previous: String,
}

// Today's sets, the current row, then rows proposed from last time.
// Proposed values: last set of today, otherwise the set of the same rank last time.
pub fn planned_set_rows(
    db: &Db,
    session_id: SessionId,
    entry_id: EntryId,
    last: Option<&LastPerformance>,
) -> Option<SetRows> {
    let entry = db.session(session_id)?.entry(entry_id)?;
    let measure_kind = db.exercise(entry.exercise_id())?.measure();
    let previous_sets: &[WorkoutSet] = last.map_or(&[], |last| &last.sets);
    let previous_text = |index: usize| {
        previous_sets
            .get(index)
            .map(|set| format_set_measure(set.measure()))
    };

    let done: Vec<DoneSetRow> = entry
        .sets()
        .iter()
        .enumerate()
        .map(|(index, set)| DoneSetRow {
            index,
            number: index + 1,
            text: format_set_measure(set.measure()),
            measure: *set.measure(),
            warmup: set.is_warmup(),
            previous: previous_text(index),
        })
        .collect();
    let done_count = done.len();
    let upcoming = previous_sets
        .iter()
        .enumerate()
        .skip(done_count + 1)
        .map(|(index, set)| UpcomingSetRow {
            number: index + 1,
            previous: format_set_measure(set.measure()),
        })
        .collect();
    let draft_source = entry
        .sets()
        .last()
        .or_else(|| previous_sets.get(done_count))
        .map(WorkoutSet::measure);

    Some(SetRows {
        done,
        current_number: done_count + 1,
        current_previous: previous_text(done_count),
        upcoming,
        next_draft: SetDraft::prefilled(measure_kind, draft_source),
    })
}

#[cfg(test)]
mod tests {
    use domain::MeasureKind;

    use super::*;
    use crate::run_session::last_performance::last_performance;
    use crate::test_support::{add_set_at, bench, done_session, start_session, weight_reps};

    fn setup() -> (Db, SessionId, Option<LastPerformance>) {
        let mut db = Db::default();
        done_session(
            &mut db,
            27,
            &[(
                bench(),
                vec![weight_reps(60, 10), weight_reps(80, 8), weight_reps(80, 6)],
            )],
        );
        let current = start_session(&mut db, &[bench()]);
        let last = last_performance(&db, &bench(), current);
        (db, current, last)
    }

    #[test]
    fn proposes_last_time_rows_before_any_set() {
        let (db, current, last) = setup();
        let rows = planned_set_rows(&db, current, EntryId::new(1), last.as_ref()).unwrap();
        assert!(rows.done.is_empty());
        assert_eq!(rows.current_number, 1);
        assert_eq!(rows.current_previous.as_deref(), Some("60 × 10"));
        assert_eq!(
            rows.upcoming,
            [
                UpcomingSetRow {
                    number: 2,
                    previous: "80 × 8".to_owned()
                },
                UpcomingSetRow {
                    number: 3,
                    previous: "80 × 6".to_owned()
                },
            ]
        );
        assert_eq!(
            rows.next_draft,
            SetDraft::prefilled(MeasureKind::WeightReps, Some(&weight_reps(60, 10)))
        );
    }

    #[test]
    fn prefills_with_the_last_set_of_today() {
        let (mut db, current, last) = setup();
        add_set_at(&mut db, current, 1, weight_reps(65, 10), 5);
        let rows = planned_set_rows(&db, current, EntryId::new(1), last.as_ref()).unwrap();
        assert_eq!(rows.done.len(), 1);
        assert_eq!(rows.done[0].text, "65 × 10");
        assert_eq!(rows.done[0].previous.as_deref(), Some("60 × 10"));
        assert_eq!(rows.current_number, 2);
        assert_eq!(rows.current_previous.as_deref(), Some("80 × 8"));
        assert_eq!(rows.upcoming.len(), 1);
        assert_eq!(
            rows.next_draft,
            SetDraft::prefilled(MeasureKind::WeightReps, Some(&weight_reps(65, 10)))
        );
    }

    #[test]
    fn starts_empty_for_a_new_exercise() {
        let mut db = Db::default();
        let current = start_session(&mut db, &[bench()]);
        let rows = planned_set_rows(&db, current, EntryId::new(1), None).unwrap();
        assert_eq!(rows.current_previous, None);
        assert!(rows.upcoming.is_empty());
        assert_eq!(rows.next_draft, SetDraft::empty(MeasureKind::WeightReps));
        assert_eq!(planned_set_rows(&db, current, EntryId::new(9), None), None);
    }
}
