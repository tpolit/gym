use chrono::NaiveDate;
use domain::{Db, SessionId};

use super::error::PlanError;

// Copies the name and exercises of a done session: no sets, no notes.
pub fn plan_from_session(
    db: &mut Db,
    source: SessionId,
    planned_for: Option<NaiveDate>,
) -> Result<SessionId, PlanError> {
    let (name, exercises) = db.session_template(source)?;
    Ok(db.plan_session(name, planned_for, &exercises)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, SessionStatus};

    use super::*;
    use crate::test_support::{
        bench, date, db_with_session_in_progress, done_session, pull_up, reps, weight_reps,
    };

    #[test]
    fn copies_name_and_exercises_in_order() {
        let mut db = Db::default();
        let source = done_session(
            &mut db,
            27,
            &[
                (bench(), vec![weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        let id = plan_from_session(&mut db, source, Some(date(31))).unwrap();
        let session = db.session(id).unwrap();
        assert_eq!(session.name().as_str(), "Push A");
        assert_eq!(session.status(), &SessionStatus::Planned);
        let exercises: Vec<_> = session
            .entries()
            .iter()
            .map(|e| e.exercise_id().clone())
            .collect();
        assert_eq!(exercises, [bench(), pull_up()]);
        assert!(!session.has_sets());
    }

    #[test]
    fn requires_a_done_source() {
        let (mut db, in_progress) = db_with_session_in_progress(&[]);
        assert_eq!(
            plan_from_session(&mut db, in_progress, None),
            Err(PlanError::Domain(DbError::NotDone))
        );
    }
}
