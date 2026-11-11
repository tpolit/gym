use domain::rest::latest_completed_set;
use domain::{Db, EntryId, SessionId};

// When opening the session: the exercise of the last completed set, otherwise the first.
pub fn entry_to_resume(db: &Db, session_id: SessionId) -> Option<EntryId> {
    let session = db.session(session_id)?;
    latest_completed_set(session)
        .map(|set| set.entry_id)
        .or_else(|| session.entries().first().map(|entry| entry.entry_id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{add_set_at, bench, db_with_session_in_progress, pull_up, reps};

    #[test]
    fn resumes_where_the_last_set_was_validated() {
        let (mut db, id) = db_with_session_in_progress(&[bench(), pull_up()]);
        assert_eq!(entry_to_resume(&db, id), Some(EntryId::new(1)));
        add_set_at(&mut db, id, 2, reps(10), 5);
        assert_eq!(entry_to_resume(&db, id), Some(EntryId::new(2)));
    }

    #[test]
    fn empty_session_has_nothing_to_resume() {
        let (db, id) = db_with_session_in_progress(&[]);
        assert_eq!(entry_to_resume(&db, id), None);
    }
}
