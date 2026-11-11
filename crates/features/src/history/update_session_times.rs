use domain::{Db, SessionId};

use super::error::{HistoryError, ensure_done};
use super::time_input::parse_time_input;

// Times are typed in the offset already recorded with the session.
pub fn update_session_times(
    db: &mut Db,
    session_id: SessionId,
    started_input: &str,
    finished_input: &str,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    let offset = db
        .session(session_id)
        .and_then(|session| session.status().started_at())
        .map(|started_at| *started_at.offset())
        .ok_or(HistoryError::InvalidTime)?;
    let started_at = parse_time_input(started_input, offset).ok_or(HistoryError::InvalidTime)?;
    let finished_at = parse_time_input(finished_input, offset).ok_or(HistoryError::InvalidTime)?;
    Ok(db.update_session_times(session_id, started_at, finished_at)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, SessionStatus};

    use super::*;
    use crate::test_support::{at, bench, done_session, weight_reps};

    #[test]
    fn corrects_the_times() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(bench(), vec![weight_reps(80, 8)])]);
        update_session_times(&mut db, id, "2026-10-28T17:30", "2026-10-28T18:45").unwrap();
        assert_eq!(
            db.session(id).unwrap().status(),
            &SessionStatus::Done {
                started_at: at(28, 17, 30),
                finished_at: at(28, 18, 45),
            }
        );
    }

    #[test]
    fn refuses_invalid_or_reversed_times() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(bench(), vec![weight_reps(80, 8)])]);
        assert_eq!(
            update_session_times(&mut db, id, "", "2026-10-28T18:45"),
            Err(HistoryError::InvalidTime)
        );
        assert_eq!(
            update_session_times(&mut db, id, "2026-10-28T19:00", "2026-10-28T18:00"),
            Err(HistoryError::Domain(DbError::FinishedBeforeStarted))
        );
    }
}
