use chrono::{DateTime, FixedOffset};
use domain::{Db, SessionId};

use super::error::RunSessionError;

// Repeat a session: same name and exercises as a done session, started right away.
pub fn repeat_session(
    db: &mut Db,
    source: SessionId,
    now: DateTime<FixedOffset>,
) -> Result<SessionId, RunSessionError> {
    let (name, exercises) = db.session_template(source)?;
    Ok(db.start_new_session(name, &exercises, now)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, SessionOrigin};

    use super::*;
    use crate::test_support::{at, bench, done_session, pull_up, reps, weight_reps};

    #[test]
    fn starts_a_copy_of_a_done_session() {
        let mut db = Db::default();
        let source = done_session(
            &mut db,
            27,
            &[
                (bench(), vec![weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        let id = repeat_session(&mut db, source, at(30, 18, 0)).unwrap();
        let session = db.session(id).unwrap();
        assert_eq!(session.name().as_str(), "Push A");
        assert_eq!(session.origin(), SessionOrigin::Adhoc);
        assert_eq!(db.session_in_progress().map(|s| s.id()), Some(id));
        assert_eq!(session.entries().len(), 2);
        assert!(!session.has_sets());
        assert_eq!(
            repeat_session(&mut db, source, at(30, 18, 5)),
            Err(RunSessionError::Domain(DbError::AnotherSessionInProgress))
        );
    }
}
