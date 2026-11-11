use domain::Db;
use serde::Deserialize;

use crate::codec::{DbCodec, DecodeError, EncodeError};
use crate::dto::v1::{self, DbV1};
use crate::mapping::{db_from_dto, db_to_dto};

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonCodec;

#[derive(Deserialize)]
struct SchemaVersionOnly {
    schema_version: u32,
}

impl DbCodec for JsonCodec {
    fn encode(&self, db: &Db) -> Result<Vec<u8>, EncodeError> {
        serde_json::to_vec(&db_to_dto(db)).map_err(|_| EncodeError::Serialization)
    }

    fn decode(&self, bytes: &[u8]) -> Result<Db, DecodeError> {
        let version: SchemaVersionOnly = serde_json::from_slice(bytes).map_err(malformed)?;
        let current = match version.schema_version {
            v1::SCHEMA_VERSION => serde_json::from_slice::<DbV1>(bytes).map_err(malformed)?,
            unsupported => return Err(DecodeError::UnsupportedSchemaVersion(unsupported)),
        };
        db_from_dto(current).map_err(DecodeError::InvalidContent)
    }
}

fn malformed(error: serde_json::Error) -> DecodeError {
    DecodeError::Malformed {
        line: error.line(),
        column: error.column(),
    }
}

#[cfg(test)]
mod tests;
