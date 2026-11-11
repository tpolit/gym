use domain::{Db, EntryId, SessionId};

use super::error::{RunSessionError, ensure_in_progress};

pub fn remove_entry(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
) -> Result<(), RunSessionError> {
    ensure_in_progress(db, session_id)?;
    Ok(db.remove_entry(session_id, entry_id)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{bench, db_with_session_in_progress, pull_up};

    #[test]
    fn removes_the_entry_and_its_sets() {
        let (mut db, id) = db_with_session_in_progress(&[bench(), pull_up()]);
        remove_entry(&mut db, id, EntryId::new(1)).unwrap();
        let entries = db.session(id).unwrap().entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].exercise_id(), &pull_up());
        assert_eq!(
            remove_entry(&mut db, id, EntryId::new(1)),
            Err(RunSessionError::Domain(DbError::EntryNotFound(
                EntryId::new(1)
            )))
        );
    }
}
