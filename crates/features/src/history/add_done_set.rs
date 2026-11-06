use domain::{Db, EntryId, SessionId, SetMeasure, WorkoutSet};

use super::error::{HistoryError, ensure_done};

// A set added afterwards has no completion time: it does not affect rest times.
pub fn add_done_set(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    measure: SetMeasure,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.add_set(session_id, entry_id, WorkoutSet::new(measure, false, None))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn adds_a_set_without_completion_time() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(bench(), vec![weight_reps(80, 8)])]);
        add_done_set(&mut db, id, EntryId::new(1), weight_reps(70, 10)).unwrap();
        let sets = db
            .session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets();
        assert_eq!(sets.len(), 2);
        assert_eq!(sets[1].completed_at(), None);
    }
}
