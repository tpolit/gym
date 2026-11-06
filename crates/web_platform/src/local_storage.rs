use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use persistence::{Storage, StorageError};
use wasm_bindgen::JsCast;

// Boundary with the browser localStorage API, replaced in memory in native tests.
pub trait TextStore {
    fn get_item(&self, key: &str) -> Result<Option<String>, StorageError>;
    fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BrowserLocalStorage;

impl BrowserLocalStorage {
    fn storage() -> Result<web_sys::Storage, StorageError> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .ok_or(StorageError::Unavailable)
    }
}

impl TextStore for BrowserLocalStorage {
    fn get_item(&self, key: &str) -> Result<Option<String>, StorageError> {
        Self::storage()?
            .get_item(key)
            .map_err(|_| StorageError::Unavailable)
    }

    fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        Self::storage()?.set_item(key, value).map_err(|error| {
            let is_quota_exceeded = error
                .dyn_ref::<web_sys::DomException>()
                .is_some_and(|exception| exception.name() == "QuotaExceededError");
            if is_quota_exceeded {
                StorageError::QuotaExceeded
            } else {
                StorageError::Unavailable
            }
        })
    }
}

// localStorage only stores text: bytes are always base64-encoded.
pub struct LocalStorage<T: TextStore = BrowserLocalStorage> {
    key: String,
    store: T,
}

impl LocalStorage<BrowserLocalStorage> {
    pub fn new(key: impl Into<String>) -> Self {
        Self::with_store(key, BrowserLocalStorage)
    }
}

impl<T: TextStore> LocalStorage<T> {
    pub fn with_store(key: impl Into<String>, store: T) -> Self {
        Self {
            key: key.into(),
            store,
        }
    }
}

impl<T: TextStore> Storage for LocalStorage<T> {
    async fn read(&self) -> Result<Option<Vec<u8>>, StorageError> {
        let Some(text) = self.store.get_item(&self.key)? else {
            return Ok(None);
        };
        BASE64
            .decode(text)
            .map(Some)
            .map_err(|_| StorageError::CorruptedContent)
    }

    async fn write(&self, bytes: &[u8]) -> Result<(), StorageError> {
        self.store.set_item(&self.key, &BASE64.encode(bytes))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use persistence::contract::check_storage_contract;
    use pollster::block_on;

    use super::*;

    #[derive(Default)]
    struct MemoryTextStore {
        items: RefCell<BTreeMap<String, String>>,
    }

    impl TextStore for MemoryTextStore {
        fn get_item(&self, key: &str) -> Result<Option<String>, StorageError> {
            Ok(self.items.borrow().get(key).cloned())
        }

        fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
            self.items
                .borrow_mut()
                .insert(key.to_owned(), value.to_owned());
            Ok(())
        }
    }

    fn storage_with_text(text: &str) -> LocalStorage<MemoryTextStore> {
        let store = MemoryTextStore::default();
        store.set_item("gym.db", text).unwrap();
        LocalStorage::with_store("gym.db", store)
    }

    #[test]
    fn satisfies_storage_contract() {
        check_storage_contract(|| LocalStorage::with_store("gym.db", MemoryTextStore::default()));
    }

    #[test]
    fn stores_bytes_as_base64_under_its_key() {
        let storage = LocalStorage::with_store("gym.db", MemoryTextStore::default());
        block_on(storage.write(b"{}")).unwrap();
        assert_eq!(
            storage.store.get_item("gym.db"),
            Ok(Some("e30=".to_owned()))
        );
    }

    #[test]
    fn text_that_is_not_base64_is_corrupted_content() {
        let storage = storage_with_text("{not base64}");
        assert_eq!(
            block_on(storage.read()),
            Err(StorageError::CorruptedContent)
        );
    }
}
