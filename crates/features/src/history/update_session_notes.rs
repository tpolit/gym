use domain::{Db, NonEmptyText, SessionId};

use super::error::{HistoryError, ensure_done};

// Blank text removes the notes.
pub fn update_session_notes(
    db: &mut Db,
    session_id: SessionId,
    notes: &str,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.set_session_notes(session_id, NonEmptyText::optional(notes))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn sets_and_clears_notes() {
        let mut db = Db::default();
        let id = done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        update_session_notes(&mut db, id, "Bonne forme").unwrap();
        assert_eq!(
            db.session(id).unwrap().notes().map(NonEmptyText::as_str),
            Some("Bonne forme")
        );
        update_session_notes(&mut db, id, " ").unwrap();
        assert_eq!(db.session(id).unwrap().notes(), None);
    }
}
