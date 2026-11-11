use domain::{Db, EntryId, SessionId};

use super::error::{PlanError, ensure_planned};

pub fn move_planned_entry(
    db: &mut Db,
    session_id: SessionId,
    entry_id: EntryId,
    new_index: usize,
) -> Result<(), PlanError> {
    ensure_planned(db, session_id)?;
    Ok(db.move_entry(session_id, entry_id, new_index)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{bench, pull_up, text};

    #[test]
    fn moves_the_entry() {
        let mut db = Db::default();
        let id = db
            .plan_session(text("Push A"), None, &[bench(), pull_up()])
            .unwrap();
        move_planned_entry(&mut db, id, EntryId::new(2), 0).unwrap();
        assert_eq!(
            db.session(id).unwrap().entries()[0].exercise_id(),
            &pull_up()
        );
        assert_eq!(
            move_planned_entry(&mut db, id, EntryId::new(2), 2),
            Err(PlanError::Domain(DbError::InvalidPosition(2)))
        );
    }
}
