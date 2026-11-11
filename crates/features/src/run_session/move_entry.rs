use domain::{Db, EntryId, SessionId};

use super::error::{RunSessionError, ensure_in_progress};

pub fn move_entry(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    new_index: usize,
) -> Result<(), RunSessionError> {
    ensure_in_progress(db, session_id)?;
    Ok(db.move_entry(session_id, entry_id, new_index)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, db_with_session_in_progress, pull_up};

    #[test]
    fn reorders_the_session() {
        let (mut db, id) = db_with_session_in_progress(&[bench(), pull_up()]);
        move_entry(&mut db, id, EntryId::new(2), 0).unwrap();
        assert_eq!(
            db.session(id).unwrap().entries()[0].exercise_id(),
            &pull_up()
        );
    }
}
