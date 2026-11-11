use domain::{Db, NonEmptyText, SessionId};

use super::error::{HistoryError, ensure_done};

pub fn rename_session(db: &mut Db, session_id: SessionId, name: &str) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    let name = NonEmptyText::new(name).map_err(|_| HistoryError::EmptyName)?;
    Ok(db.rename_session(session_id, name)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn renames_a_done_session() {
        let mut db = Db::default();
        let id = done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        rename_session(&mut db, id, "Pecs").unwrap();
        assert_eq!(db.session(id).unwrap().name().as_str(), "Pecs");
        assert_eq!(
            rename_session(&mut db, id, ""),
            Err(HistoryError::EmptyName)
        );
    }
}
