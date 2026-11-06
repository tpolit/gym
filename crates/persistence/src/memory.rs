use std::cell::RefCell;

use crate::storage::{Storage, StorageError};

#[derive(Debug, Default)]
pub struct MemoryStorage {
    content: RefCell<Option<Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content(bytes: Vec<u8>) -> Self {
        Self {
            content: RefCell::new(Some(bytes)),
        }
    }
}

impl Storage for MemoryStorage {
    async fn read(&self) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.content.borrow().clone())
    }

    async fn write(&self, bytes: &[u8]) -> Result<(), StorageError> {
        self.content.replace(Some(bytes.to_vec()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::check_storage_contract;

    #[test]
    fn satisfies_storage_contract() {
        check_storage_contract(MemoryStorage::new);
    }

    #[test]
    fn starts_with_given_content() {
        let storage = MemoryStorage::with_content(b"data".to_vec());
        assert_eq!(
            pollster::block_on(storage.read()),
            Ok(Some(b"data".to_vec()))
        );
    }
}
