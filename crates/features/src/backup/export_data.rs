use chrono::NaiveDate;
use domain::Db;
use persistence::DbCodec;

use super::error::BackupError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFile {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

// The view triggers the download.
pub fn export_data<C: DbCodec>(
    db: &Db,
    codec: &C,
    today: NaiveDate,
) -> Result<ExportFile, BackupError> {
    Ok(ExportFile {
        file_name: format!("gym-{today}.json"),
        bytes: codec.encode(db).map_err(BackupError::Encode)?,
    })
}

#[cfg(test)]
mod tests {
    use persistence::JsonCodec;

    use super::*;
    use crate::test_support::{bench, date, done_session, weight_reps};

    #[test]
    fn exports_a_dated_file_that_decodes_back() {
        let mut db = Db::default();
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let file = export_data(&db, &JsonCodec, date(30)).unwrap();
        assert_eq!(file.file_name, "gym-2026-10-30.json");
        assert_eq!(JsonCodec.decode(&file.bytes), Ok(db));
    }
}
