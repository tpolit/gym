use chrono::{DateTime, FixedOffset};
use domain::{Db, NonEmptyText, SessionId};

use super::error::RunSessionError;

pub fn start_new_session(
    db: &mut Db,
    name: &str,
    now: DateTime<FixedOffset>,
) -> Result<SessionId, RunSessionError> {
    let name = NonEmptyText::new(name).map_err(|_| RunSessionError::EmptyName)?;
    Ok(db.start_new_session(name, &[], now)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, SessionOrigin, SessionStatus};

    use super::*;
    use crate::test_support::at;

    #[test]
    fn starts_an_empty_adhoc_session() {
        let mut db = Db::default();
        let id = start_new_session(&mut db, " Push A ", at(30, 18, 0)).unwrap();
        let session = db.session(id).unwrap();
        assert_eq!(session.name().as_str(), "Push A");
        assert_eq!(session.origin(), SessionOrigin::Adhoc);
        assert_eq!(
            session.status(),
            &SessionStatus::InProgress {
                started_at: at(30, 18, 0)
            }
        );
        assert!(session.entries().is_empty());
    }

    #[test]
    fn refuses_empty_name_or_second_session() {
        let mut db = Db::default();
        assert_eq!(
            start_new_session(&mut db, "  ", at(30, 18, 0)),
            Err(RunSessionError::EmptyName)
        );
        start_new_session(&mut db, "Push A", at(30, 18, 0)).unwrap();
        assert_eq!(
            start_new_session(&mut db, "Jambes", at(30, 18, 5)),
            Err(RunSessionError::Domain(DbError::AnotherSessionInProgress))
        );
    }
}
