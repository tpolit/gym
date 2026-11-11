use chrono::{DateTime, FixedOffset};
use domain::{Db, EntryId, SessionId, SetMeasure, WorkoutSet};

use super::error::{RunSessionError, ensure_in_progress};

// The completion time starts the rest timer.
pub fn add_set(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    measure: SetMeasure,
    now: DateTime<FixedOffset>,
) -> Result<(), RunSessionError> {
    ensure_in_progress(db, session_id)?;
    let set = WorkoutSet::new(measure, false, Some(now));
    Ok(db.add_set(session_id, entry_id, set)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, MeasureKind};

    use super::*;
    use crate::test_support::{at, bench, db_with_session_in_progress, reps, weight_reps};

    #[test]
    fn records_the_set_with_its_completion_time() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        add_set(
            &mut db,
            id,
            EntryId::new(1),
            weight_reps(80, 8),
            at(30, 18, 5),
        )
        .unwrap();
        let sets = db
            .session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets();
        assert_eq!(sets.len(), 1);
        assert_eq!(sets[0].measure(), &weight_reps(80, 8));
        assert!(!sets[0].is_warmup());
        assert_eq!(sets[0].completed_at(), Some(at(30, 18, 5)));
    }

    #[test]
    fn refuses_a_measure_of_another_kind() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        assert_eq!(
            add_set(&mut db, id, EntryId::new(1), reps(10), at(30, 18, 5)),
            Err(RunSessionError::Domain(DbError::MeasureMismatch {
                expected: MeasureKind::WeightReps,
                found: MeasureKind::Reps,
            }))
        );
    }
}
