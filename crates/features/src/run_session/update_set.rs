use domain::{Db, EntryId, SessionId, SetMeasure};

use super::error::{RunSessionError, ensure_in_progress};

pub fn update_set(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    index: usize,
    measure: SetMeasure,
    warmup: bool,
) -> Result<(), RunSessionError> {
    ensure_in_progress(db, session_id)?;
    Ok(db.update_set(session_id, entry_id, index, measure, warmup)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{add_set_at, at, bench, db_with_session_in_progress, weight_reps};

    #[test]
    fn marks_a_set_as_warmup_and_keeps_its_time() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        add_set_at(&mut db, id, 1, weight_reps(60, 10), 5);
        update_set(&mut db, id, EntryId::new(1), 0, weight_reps(60, 12), true).unwrap();
        let set = &db
            .session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets()[0];
        assert_eq!(set.measure(), &weight_reps(60, 12));
        assert!(set.is_warmup());
        assert_eq!(set.completed_at(), Some(at(30, 18, 5)));
    }
}
