use domain::{Db, EntryId, ExerciseId, SessionId};

use super::error::{HistoryError, ensure_done};

pub fn add_done_entry(
    db: &mut Db,
    session_id: SessionId,
    exercise_id: ExerciseId,
) -> Result<EntryId, HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.add_entry(session_id, exercise_id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, weight_reps};

    #[test]
    fn adds_a_forgotten_exercise() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(bench(), vec![weight_reps(80, 8)])]);
        assert_eq!(add_done_entry(&mut db, id, pull_up()), Ok(EntryId::new(2)));
    }
}
