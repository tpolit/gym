use domain::{Db, SessionId};

use super::error::{PlanError, ensure_planned};

pub fn delete_planned_session(db: &mut Db, session_id: SessionId) -> Result<(), PlanError> {
    ensure_planned(db, session_id)?;
    Ok(db.delete_session(session_id)?)
}

#[cfg(test)]
mod tests {
    use domain::DbError;

    use super::*;
    use crate::test_support::{db_with_session_in_progress, text};

    #[test]
    fn deletes_only_planned_sessions() {
        let (mut db, in_progress) = db_with_session_in_progress(&[]);
        let planned = db.plan_session(text("Jambes"), None, &[]).unwrap();
        delete_planned_session(&mut db, planned).unwrap();
        assert!(db.session(planned).is_none());
        assert_eq!(
            delete_planned_session(&mut db, in_progress),
            Err(PlanError::Domain(DbError::NotPlanned))
        );
    }
}
