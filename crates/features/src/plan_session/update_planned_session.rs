use chrono::NaiveDate;
use domain::{Db, NonEmptyText, SessionId};

use super::error::{PlanError, ensure_planned};

pub fn update_planned_session(
    db: &mut Db,
    session_id: SessionId,
    name: &str,
    planned_for: Option<NaiveDate>,
) -> Result<(), PlanError> {
    ensure_planned(db, session_id)?;
    let name = NonEmptyText::new(name).map_err(|_| PlanError::EmptyName)?;
    db.rename_session(session_id, name)?;
    Ok(db.set_session_planned_for(session_id, planned_for)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{date, text};

    #[test]
    fn renames_and_reschedules() {
        let mut db = Db::default();
        let id = db.plan_session(text("Jambes"), None, &[]).unwrap();
        update_planned_session(&mut db, id, "Jambes B", Some(date(31))).unwrap();
        let session = db.session(id).unwrap();
        assert_eq!(session.name().as_str(), "Jambes B");
        assert_eq!(session.planned_for(), Some(date(31)));
    }

    #[test]
    fn empty_name_changes_nothing() {
        let mut db = Db::default();
        let id = db.plan_session(text("Jambes"), None, &[]).unwrap();
        let before = db.clone();
        assert_eq!(
            update_planned_session(&mut db, id, " ", Some(date(31))),
            Err(PlanError::EmptyName)
        );
        assert_eq!(db, before);
    }
}
