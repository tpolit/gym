use domain::{Db, EntryId, SessionId};

use super::error::{HistoryError, ensure_done};

pub fn remove_done_entry(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.remove_entry(session_id, entry_id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, weight_reps};

    #[test]
    fn removes_an_exercise_and_its_sets() {
        let mut db = Db::default();
        let id = done_session(
            &mut db,
            28,
            &[
                (bench(), vec![weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        remove_done_entry(&mut db, id, EntryId::new(1)).unwrap();
        assert_eq!(db.session(id).unwrap().entries().len(), 1);
    }
}
