use domain::{Db, SessionId};

use super::error::{HistoryError, ensure_done};

// The view asks for confirmation.
pub fn delete_session(db: &mut Db, session_id: SessionId) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.delete_session(session_id)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{bench, done_session, start_session, weight_reps};

    #[test]
    fn deletes_only_done_sessions() {
        let mut db = Db::default();
        let done = done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let current = start_session(&mut db, &[]);
        delete_session(&mut db, done).unwrap();
        assert!(db.session(done).is_none());
        assert_eq!(
            delete_session(&mut db, current),
            Err(HistoryError::Domain(DbError::NotDone))
        );
    }
}
