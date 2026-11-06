use domain::{Db, EntryId, SessionId};

use super::error::{PlanError, ensure_planned};

pub fn remove_planned_entry(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
) -> Result<(), PlanError> {
    ensure_planned(db, session_id)?;
    Ok(db.remove_entry(session_id, entry_id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, pull_up, text};

    #[test]
    fn removes_the_entry() {
        let mut db = Db::default();
        let id = db
            .plan_session(text("Push A"), None, &[bench(), pull_up()])
            .unwrap();
        remove_planned_entry(&mut db, id, EntryId::new(1)).unwrap();
        let entries = db.session(id).unwrap().entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].exercise_id(), &pull_up());
    }
}
