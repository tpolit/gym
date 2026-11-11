use domain::{Db, DbError, SessionId, SessionStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryError {
    EmptyName,
    InvalidTime,
    Domain(DbError),
}

impl From<DbError> for HistoryError {
    fn from(error: DbError) -> Self {
        Self::Domain(error)
    }
}

pub fn history_error_message(error: &HistoryError) -> &'static str {
    match error {
        HistoryError::EmptyName => "Donnez un nom à la séance.",
        HistoryError::InvalidTime => "Horaire invalide.",
        HistoryError::Domain(DbError::FinishedBeforeStarted) => "La fin doit être après le début.",
        HistoryError::Domain(DbError::MeasureMismatch { .. }) => {
            "Cette série ne correspond pas au type de mesure de l'exercice."
        }
        HistoryError::Domain(DbError::ExerciseArchived(_)) => "Cet exercice est archivé.",
        HistoryError::Domain(DbError::NotDone | DbError::SessionNotFound(_)) => {
            "Cette séance n'est plus dans l'historique."
        }
        HistoryError::Domain(_) => "Action impossible.",
    }
}

pub(super) fn ensure_done(db: &Db, session_id: SessionId) -> Result<(), HistoryError> {
    let session = db
        .session(session_id)
        .ok_or(DbError::SessionNotFound(session_id))?;
    match session.status() {
        SessionStatus::Done { .. } => Ok(()),
        SessionStatus::Planned | SessionStatus::InProgress { .. } => Err(DbError::NotDone.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, start_session, weight_reps};

    #[test]
    fn only_done_sessions_are_accepted() {
        let mut db = Db::default();
        let done = done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let current = start_session(&mut db, &[]);
        assert_eq!(ensure_done(&db, done), Ok(()));
        assert_eq!(
            ensure_done(&db, current),
            Err(HistoryError::Domain(DbError::NotDone))
        );
        assert_eq!(
            history_error_message(&HistoryError::EmptyName),
            "Donnez un nom à la séance."
        );
    }
}
