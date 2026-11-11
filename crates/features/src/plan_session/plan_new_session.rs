use chrono::NaiveDate;
use domain::{Db, ExerciseId, NonEmptyText, SessionId};

use super::error::PlanError;

pub fn plan_session(
    db: &mut Db,
    name: &str,
    planned_for: Option<NaiveDate>,
    exercises: &[ExerciseId],
) -> Result<SessionId, PlanError> {
    let name = NonEmptyText::new(name).map_err(|_| PlanError::EmptyName)?;
    Ok(db.plan_session(name, planned_for, exercises)?)
}

#[cfg(test)]
mod tests {
    use domain::{SessionOrigin, SessionStatus};

    use super::*;
    use crate::test_support::{bench, date};

    #[test]
    fn plans_a_named_session() {
        let mut db = Db::default();
        let id = plan_session(&mut db, " Jambes ", Some(date(31)), &[bench()]).unwrap();
        let session = db.session(id).unwrap();
        assert_eq!(session.name().as_str(), "Jambes");
        assert_eq!(session.origin(), SessionOrigin::Planned);
        assert_eq!(session.status(), &SessionStatus::Planned);
        assert_eq!(session.planned_for(), Some(date(31)));
        assert_eq!(session.entries().len(), 1);
        assert_eq!(
            plan_session(&mut db, "", None, &[]),
            Err(PlanError::EmptyName)
        );
    }
}
