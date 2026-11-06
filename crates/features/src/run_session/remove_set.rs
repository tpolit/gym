use domain::{Db, EntryId, SessionId};

use super::error::{RunSessionError, ensure_in_progress};

pub fn remove_set(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    index: usize,
) -> Result<(), RunSessionError> {
    ensure_in_progress(db, session_id)?;
    Ok(db.remove_set(session_id, entry_id, index)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{add_set_at, bench, db_with_session_in_progress, weight_reps};

    #[test]
    fn removes_the_set_at_the_index() {
        let (mut db, id) = db_with_session_in_progress(&[bench()]);
        add_set_at(&mut db, id, 1, weight_reps(60, 10), 5);
        add_set_at(&mut db, id, 1, weight_reps(80, 8), 8);
        remove_set(&mut db, id, EntryId::new(1), 0).unwrap();
        let sets = db
            .session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets();
        assert_eq!(sets.len(), 1);
        assert_eq!(sets[0].measure(), &weight_reps(80, 8));
    }
}
