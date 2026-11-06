use domain::Db;

use crate::codec::{DbCodec, DecodeError, EncodeError};
use crate::storage::{Storage, StorageError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Storage(StorageError),
    Decode(DecodeError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError {
    Encode(EncodeError),
    Storage(StorageError),
}

pub struct DbRepository<S: Storage, C: DbCodec> {
    storage: S,
    codec: C,
}

impl<S: Storage, C: DbCodec> DbRepository<S, C> {
    pub const fn new(storage: S, codec: C) -> Self {
        Self { storage, codec }
    }

    pub async fn load(&self) -> Result<Option<Db>, LoadError> {
        let Some(bytes) = self.storage.read().await.map_err(LoadError::Storage)? else {
            return Ok(None);
        };
        self.codec
            .decode(&bytes)
            .map(Some)
            .map_err(LoadError::Decode)
    }

    pub async fn save(&self, db: &Db) -> Result<(), SaveError> {
        let bytes = self.codec.encode(db).map_err(SaveError::Encode)?;
        self.storage.write(&bytes).await.map_err(SaveError::Storage)
    }
}

#[cfg(test)]
mod tests {
    use domain::{NonEmptyText, WeeklyGoal};
    use pollster::block_on;

    use super::*;
    use crate::json::JsonCodec;
    use crate::memory::MemoryStorage;

    struct UnavailableStorage;

    impl Storage for UnavailableStorage {
        async fn read(&self) -> Result<Option<Vec<u8>>, StorageError> {
            Err(StorageError::Unavailable)
        }

        async fn write(&self, _: &[u8]) -> Result<(), StorageError> {
            Err(StorageError::QuotaExceeded)
        }
    }

    fn sample_db() -> Db {
        let mut db = Db::default();
        db.set_weekly_goal(WeeklyGoal::new(3).ok());
        db.plan_session(NonEmptyText::new("Push A").unwrap(), None, &[])
            .unwrap();
        db
    }

    #[test]
    fn first_launch_loads_nothing() {
        let repository = DbRepository::new(MemoryStorage::new(), JsonCodec);
        assert_eq!(block_on(repository.load()), Ok(None));
    }

    #[test]
    fn saved_db_is_loaded_back() {
        let repository = DbRepository::new(MemoryStorage::new(), JsonCodec);
        block_on(repository.save(&sample_db())).unwrap();
        assert_eq!(block_on(repository.load()), Ok(Some(sample_db())));
    }

    #[test]
    fn unreadable_content_is_a_decode_error() {
        let repository = DbRepository::new(MemoryStorage::with_content(b"{".to_vec()), JsonCodec);
        assert!(matches!(
            block_on(repository.load()),
            Err(LoadError::Decode(DecodeError::Malformed { .. }))
        ));
    }

    #[test]
    fn storage_failures_are_reported() {
        let repository = DbRepository::new(UnavailableStorage, JsonCodec);
        assert_eq!(
            block_on(repository.load()),
            Err(LoadError::Storage(StorageError::Unavailable))
        );
        assert_eq!(
            block_on(repository.save(&sample_db())),
            Err(SaveError::Storage(StorageError::QuotaExceeded))
        );
    }
}
