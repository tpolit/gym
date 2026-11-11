#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    Unavailable,
    QuotaExceeded,
    CorruptedContent,
}

// Contract shared by every implementation (checked by `contract::check_storage_contract`):
// `read` returns `Ok(None)` until something is written, then exactly the last bytes written,
// whatever they are (empty and non-UTF-8 included).
pub trait Storage {
    fn read(&self) -> impl Future<Output = Result<Option<Vec<u8>>, StorageError>>;
    fn write(&self, bytes: &[u8]) -> impl Future<Output = Result<(), StorageError>>;
}
