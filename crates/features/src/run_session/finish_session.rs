use chrono::{DateTime, FixedOffset};
use domain::{Db, SessionId};

use super::error::RunSessionError;

// Refused without any set: the view then offers to delete the session.
pub fn finish_session(
    db: &mut Db,
    session_id: SessionId,
    now: DateTime<FixedOffset>,
) -> Result<(), RunSessionError> {
    Ok(db.finish_session(session_id, now)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, SessionStatus};

    use super::*;
    use crate::test_support::{
        add_set_at, at, bench, db_with_session_in_progress, pull_up, weight_reps,
    };

    #[test]
    fn finishes_and_drops_entries_without_sets() {
        let (mut db, id) = db_with_session_in_progress(&[bench(), pull_up()]);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 5);
        finish_session(&mut db, id, at(30, 19, 0)).unwrap();
        let session = db.session(id).unwrap();
        assert!(matches!(session.status(), SessionStatus::Done { .. }));
        assert_eq!(session.entries().len(), 1);
    }

    #[test]
    fn refuses_a_session_without_sets() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        assert_eq!(
            finish_session(&mut db, id, at(30, 19, 0)),
            Err(RunSessionError::Domain(DbError::FinishWithoutSets))
        );
    }
}
