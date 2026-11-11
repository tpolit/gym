use domain::{Db, EntryId, SessionId, SetMeasure};

use super::error::{HistoryError, ensure_done};

pub fn update_done_set(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    index: usize,
    measure: SetMeasure,
    warmup: bool,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.update_set(session_id, entry_id, index, measure, warmup)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn fixes_a_typo() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(bench(), vec![weight_reps(800, 8)])]);
        update_done_set(&mut db, id, EntryId::new(1), 0, weight_reps(80, 8), false).unwrap();
        let set = &db
            .session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets()[0];
        assert_eq!(set.measure(), &weight_reps(80, 8));
        assert!(set.completed_at().is_some());
    }
}
