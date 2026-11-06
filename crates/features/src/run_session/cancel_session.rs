use domain::{Cancelled, Db, SessionId};

use super::error::RunSessionError;

pub fn cancel_session(db: &mut Db, session_id: SessionId) -> Result<Cancelled, RunSessionError> {
    Ok(db.cancel_session(session_id)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{add_set_at, bench, db_with_session_in_progress, weight_reps};

    #[test]
    fn adhoc_session_without_sets_is_deleted() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        assert_eq!(cancel_session(&mut db, id), Ok(Cancelled::Deleted));
        assert!(db.sessions().is_empty());
    }

    #[test]
    fn session_with_sets_cannot_be_cancelled() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 5);
        assert_eq!(
            cancel_session(&mut db, id),
            Err(RunSessionError::Domain(DbError::CancelWithSets))
        );
    }
}
