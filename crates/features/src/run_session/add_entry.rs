use domain::{Db, EntryId, ExerciseId, SessionId};

use super::error::{RunSessionError, ensure_in_progress};

pub fn add_entry(
    db: &mut Db,
    session_id: SessionId,
    exercise_id: ExerciseId,
) -> Result<EntryId, RunSessionError> {
    ensure_in_progress(db, session_id)?;
    Ok(db.add_entry(session_id, exercise_id)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{bench, db_with_session_in_progress, done_session, weight_reps};

    #[test]
    fn adds_the_exercise_at_the_end() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        let entry = add_entry(&mut db, id, bench()).unwrap();
        assert_eq!(entry, EntryId::new(2));
        assert_eq!(db.session(id).unwrap().entries().len(), 2);
    }

    #[test]
    fn refuses_a_session_that_is_not_in_progress() {
        let mut db = Db::default();
        let done = done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        assert_eq!(
            add_entry(&mut db, done, bench()),
            Err(RunSessionError::Domain(DbError::NotInProgress))
        );
    }
}
