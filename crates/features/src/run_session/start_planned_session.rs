use chrono::{DateTime, FixedOffset};
use domain::{Db, SessionId};

use super::error::RunSessionError;

pub fn start_planned_session(
    db: &mut Db,
    session_id: SessionId,
    now: DateTime<FixedOffset>,
) -> Result<(), RunSessionError> {
    Ok(db.start_planned_session(session_id, now)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, SessionStatus};

    use super::*;
    use crate::test_support::{at, bench, start_session, text};

    #[test]
    fn starts_the_planned_session() {
        let mut db = Db::default();
        let id = db.plan_session(text("Push A"), None, &[bench()]).unwrap();
        start_planned_session(&mut db, id, at(30, 18, 0)).unwrap();
        assert_eq!(
            db.session(id).unwrap().status(),
            &SessionStatus::InProgress {
                started_at: at(30, 18, 0)
            }
        );
    }

    #[test]
    fn refuses_when_another_session_is_in_progress() {
        let mut db = Db::default();
        let planned = db.plan_session(text("Push A"), None, &[]).unwrap();
        start_session(&mut db, &[]);
        assert_eq!(
            start_planned_session(&mut db, planned, at(30, 18, 5)),
            Err(RunSessionError::Domain(DbError::AnotherSessionInProgress))
        );
    }
}
