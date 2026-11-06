use domain::{Db, DbError, SessionId, SessionStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    EmptyName,
    Domain(DbError),
}

impl From<DbError> for PlanError {
    fn from(error: DbError) -> Self {
        Self::Domain(error)
    }
}

pub fn plan_error_message(error: &PlanError) -> &'static str {
    match error {
        PlanError::EmptyName => "Donnez un nom à la séance.",
        PlanError::Domain(DbError::NotPlanned | DbError::SessionNotFound(_)) => {
            "Cette séance n'est plus planifiée."
        }
        PlanError::Domain(DbError::NotDone) => "Seule une séance terminée peut servir de modèle.",
        PlanError::Domain(DbError::ExerciseArchived(_)) => "Cet exercice est archivé.",
        PlanError::Domain(_) => "Action impossible.",
    }
}

pub(super) fn ensure_planned(db: &Db, session_id: SessionId) -> Result<(), PlanError> {
    let session = db
        .session(session_id)
        .ok_or(DbError::SessionNotFound(session_id))?;
    match session.status() {
        SessionStatus::Planned => Ok(()),
        SessionStatus::InProgress { .. } | SessionStatus::Done { .. } => {
            Err(DbError::NotPlanned.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{db_with_session_in_progress, text};

    #[test]
    fn only_planned_sessions_are_accepted() {
        let (mut db, in_progress) = db_with_session_in_progress(&[]);
        let planned = db.plan_session(text("Jambes"), None, &[]).unwrap();
        assert_eq!(ensure_planned(&db, planned), Ok(()));
        assert_eq!(
            ensure_planned(&db, in_progress),
            Err(PlanError::Domain(DbError::NotPlanned))
        );
        assert_eq!(
            plan_error_message(&PlanError::Domain(DbError::NotPlanned)),
            "Cette séance n'est plus planifiée."
        );
    }
}
