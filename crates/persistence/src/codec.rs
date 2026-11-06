use domain::Db;

use crate::mapping::MappingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    Serialization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Malformed { line: usize, column: usize },
    UnsupportedSchemaVersion(u32),
    InvalidContent(MappingError),
}

// Contract (checked by `contract::check_codec_contract`): `decode(encode(db)) == db`;
// `decode` accepts every published schema version, applies migrations and
// checks every domain invariant.
pub trait DbCodec {
    fn encode(&self, db: &Db) -> Result<Vec<u8>, EncodeError>;
    fn decode(&self, bytes: &[u8]) -> Result<Db, DecodeError>;
}
