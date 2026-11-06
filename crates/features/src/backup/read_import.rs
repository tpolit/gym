use domain::{Db, SessionStatus};
use persistence::DbCodec;

use super::error::BackupError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPreview {
    pub db: Db,
    pub summary: String,
}

// First step: decode and check every invariant before anything is replaced.
pub fn read_import<C: DbCodec>(bytes: &[u8], codec: &C) -> Result<ImportPreview, BackupError> {
    let db = codec.decode(bytes).map_err(BackupError::Decode)?;
    Ok(ImportPreview {
        summary: data_summary(&db),
        db,
    })
}

pub fn data_summary(db: &Db) -> String {
    let done = db
        .sessions()
        .iter()
        .filter(|session| matches!(session.status(), SessionStatus::Done { .. }))
        .count();
    let custom = db.custom_exercises().len();
    let plural = |count: usize| if count > 1 { "s" } else { "" };
    format!(
        "{done} séance{} terminée{}, {custom} exercice{} perso",
        plural(done),
        plural(done),
        plural(custom)
    )
}

#[cfg(test)]
mod tests {
    use domain::MeasureKind;
    use persistence::{DecodeError, JsonCodec};

    use super::*;
    use crate::test_support::{bench, custom_info, done_session, weight_reps};

    #[test]
    fn previews_a_valid_backup() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        db.add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        let bytes = JsonCodec.encode(&db).unwrap();
        let preview = read_import(&bytes, &JsonCodec).unwrap();
        assert_eq!(preview.db, db);
        assert_eq!(preview.summary, "2 séances terminées, 1 exercice perso");
    }

    #[test]
    fn rejects_files_that_are_not_backups() {
        assert!(matches!(
            read_import(b"not json", &JsonCodec),
            Err(BackupError::Decode(DecodeError::Malformed { .. }))
        ));
        assert_eq!(
            data_summary(&Db::default()),
            "0 séance terminée, 0 exercice perso"
        );
    }
}
