use domain::Db;
use persistence::LoadError;

#[derive(Debug, Clone)]
pub struct AppCore {
    db: Db,
    load_error: Option<LoadError>,
}

impl AppCore {
    pub fn from_load(result: Result<Option<Db>, LoadError>) -> Self {
        let (db, load_error) = match result {
            Ok(loaded) => (loaded.unwrap_or_default(), None),
            Err(error) => (Db::default(), Some(error)),
        };
        Self { db, load_error }
    }

    pub const fn db(&self) -> &Db {
        &self.db
    }

    pub const fn load_error(&self) -> Option<&LoadError> {
        self.load_error.as_ref()
    }

    // After a failed load, saving would overwrite data that might still be recoverable.
    pub const fn can_save(&self) -> bool {
        self.load_error.is_none()
    }

    pub fn execute<T, E>(&mut self, command: impl FnOnce(&mut Db) -> Result<T, E>) -> Result<T, E> {
        command(&mut self.db)
    }
}

#[cfg(test)]
mod tests {
    use domain::{DbError, NonEmptyText, SessionId};
    use persistence::StorageError;

    use super::*;

    fn db_with_planned_session() -> Db {
        let mut db = Db::default();
        db.plan_session(NonEmptyText::new("Push A").unwrap(), None, &[])
            .unwrap();
        db
    }

    fn core_with_planned_session() -> AppCore {
        AppCore::from_load(Ok(Some(db_with_planned_session())))
    }

    fn delete_first_session(db: &mut Db) -> Result<(), DbError> {
        db.delete_session(SessionId::new(1))
    }

    #[test]
    fn first_launch_starts_with_empty_db() {
        let core = AppCore::from_load(Ok(None));
        assert_eq!(core.db(), &Db::default());
        assert!(core.can_save());
        assert_eq!(core.load_error(), None);
    }

    #[test]
    fn loaded_db_is_kept() {
        let core = core_with_planned_session();
        assert_eq!(core.db(), &db_with_planned_session());
        assert!(core.can_save());
    }

    #[test]
    fn failed_load_forbids_saving() {
        let error = LoadError::Storage(StorageError::Unavailable);
        let core = AppCore::from_load(Err(error.clone()));
        assert!(!core.can_save());
        assert_eq!(core.load_error(), Some(&error));
    }

    #[test]
    fn execute_applies_the_command() {
        let mut core = core_with_planned_session();
        assert_eq!(core.execute(delete_first_session), Ok(()));
        assert!(core.db().sessions().is_empty());
    }

    #[test]
    fn failed_command_leaves_the_db_unchanged() {
        let mut core = AppCore::from_load(Ok(None));
        assert!(core.execute(delete_first_session).is_err());
        assert_eq!(core.db(), &Db::default());
    }
}
