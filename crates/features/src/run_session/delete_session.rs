use domain::{Db, SessionId};

use super::error::{RunSessionError, ensure_in_progress};

// The view asks for confirmation.
pub fn delete_session(db: &mut Db, session_id: SessionId) -> Result<(), RunSessionError> {
    ensure_in_progress(db, session_id)?;
    Ok(db.delete_session(session_id)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{
        add_set_at, bench, db_with_session_in_progress, done_session, weight_reps,
    };

    #[test]
    fn deletes_the_session_in_progress_with_its_sets() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 5);
        delete_session(&mut db, id).unwrap();
        assert!(db.sessions().is_empty());
    }

    #[test]
    fn does_not_delete_a_done_session() {
        let mut db = Db::default();
        let done = done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        assert_eq!(
            delete_session(&mut db, done),
            Err(RunSessionError::Domain(DbError::NotInProgress))
        );
    }
}
