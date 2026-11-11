use domain::{Db, EntryId, SessionId};

use super::error::{HistoryError, ensure_done};

pub fn move_done_entry(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    new_index: usize,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.move_entry(session_id, entry_id, new_index)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, weight_reps};

    #[test]
    fn reorders_the_exercises() {
        let mut db = Db::default();
        let id = done_session(
            &mut db,
            28,
            &[
                (bench(), vec![weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        move_done_entry(&mut db, id, EntryId::new(2), 0).unwrap();
        assert_eq!(
            db.session(id).unwrap().entries()[0].exercise_id(),
            &pull_up()
        );
    }
}
