use domain::{Db, EntryId, ExerciseId, SessionId};

use super::error::{PlanError, ensure_planned};

pub fn add_planned_exercise(
    db: &mut Db,
    session_id: SessionId,
    exercise_id: ExerciseId,
) -> Result<EntryId, PlanError> {
    ensure_planned(db, session_id)?;
    Ok(db.add_entry(session_id, exercise_id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, text};

    #[test]
    fn appends_the_exercise() {
        let mut db = Db::default();
        let id = db.plan_session(text("Push A"), None, &[bench()]).unwrap();
        assert_eq!(
            add_planned_exercise(&mut db, id, bench()),
            Ok(EntryId::new(2))
        );
        assert_eq!(db.session(id).unwrap().entries().len(), 2);
    }
}
