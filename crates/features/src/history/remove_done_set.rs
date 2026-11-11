use domain::{Db, EntryId, SessionId};

use super::error::{HistoryError, ensure_done};

pub fn remove_done_set(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    index: usize,
) -> Result<(), HistoryError> {
    ensure_done(db, session_id)?;
    Ok(db.remove_set(session_id, entry_id, index)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn removes_the_set() {
        let mut db = Db::default();
        let id = done_session(
            &mut db,
            28,
            &[(bench(), vec![weight_reps(80, 8), weight_reps(80, 6)])],
        );
        remove_done_set(&mut db, id, EntryId::new(1), 0).unwrap();
        let sets = db
            .session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets();
        assert_eq!(sets.len(), 1);
        assert_eq!(sets[0].measure(), &weight_reps(80, 6));
    }
}
