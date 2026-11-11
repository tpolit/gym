use domain::{Db, DbError, SessionId, SessionStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunSessionError {
    EmptyName,
    Domain(DbError),
}

impl From<DbError> for RunSessionError {
    fn from(error: DbError) -> Self {
        Self::Domain(error)
    }
}

pub fn run_session_error_message(error: &RunSessionError) -> &'static str {
    match error {
        RunSessionError::EmptyName => "Donnez un nom à la séance.",
        RunSessionError::Domain(DbError::AnotherSessionInProgress) => {
            "Une séance est déjà en cours."
        }
        RunSessionError::Domain(DbError::FinishWithoutSets) => {
            "Aucune série enregistrée : la séance ne peut pas être terminée."
        }
        RunSessionError::Domain(DbError::CancelWithSets) => {
            "Des séries sont enregistrées : terminez ou supprimez la séance."
        }
        RunSessionError::Domain(DbError::NotInProgress | DbError::SessionNotFound(_)) => {
            "Cette séance n'est plus en cours."
        }
        RunSessionError::Domain(DbError::ExerciseArchived(_)) => "Cet exercice est archivé.",
        RunSessionError::Domain(_) => "Action impossible.",
    }
}

pub(super) fn ensure_in_progress(db: &Db, session_id: SessionId) -> Result<(), RunSessionError> {
    let session = db
        .session(session_id)
        .ok_or(DbError::SessionNotFound(session_id))?;
    match session.status() {
        SessionStatus::InProgress { .. } => Ok(()),
        SessionStatus::Planned | SessionStatus::Done { .. } => Err(DbError::NotInProgress.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, start_session, weight_reps};

    #[test]
    fn only_the_session_in_progress_is_accepted() {
        let mut db = Db::default();
        let done = done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        let current = start_session(&mut db, &[]);
        assert_eq!(ensure_in_progress(&db, current), Ok(()));
        assert_eq!(
            ensure_in_progress(&db, done),
            Err(RunSessionError::Domain(DbError::NotInProgress))
        );
        assert_eq!(
            ensure_in_progress(&db, SessionId::new(99)),
            Err(RunSessionError::Domain(DbError::SessionNotFound(
                SessionId::new(99)
            )))
        );
    }

    #[test]
    fn messages_are_in_french() {
        assert_eq!(
            run_session_error_message(&RunSessionError::Domain(DbError::AnotherSessionInProgress)),
            "Une séance est déjà en cours."
        );
        assert_eq!(
            run_session_error_message(&RunSessionError::Domain(DbError::IdOverflow)),
            "Action impossible."
        );
    }
}
