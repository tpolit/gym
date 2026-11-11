use chrono::NaiveDate;
use persistence::{DecodeError, LoadError, StorageError};

// The raw content is offered for download so that unreadable data can still be recovered.
pub fn raw_export_file_name(today: NaiveDate) -> String {
    format!("gym-brut-{today}.json")
}

pub fn load_failure_message(error: &LoadError) -> &'static str {
    match error {
        LoadError::Storage(StorageError::Unavailable) => {
            "Le stockage du navigateur est inaccessible."
        }
        LoadError::Storage(StorageError::QuotaExceeded) => "Le stockage du navigateur est plein.",
        LoadError::Decode(DecodeError::UnsupportedSchemaVersion(_)) => {
            "Les données ont été enregistrées par une version plus récente de l'application."
        }
        LoadError::Storage(StorageError::CorruptedContent)
        | LoadError::Decode(DecodeError::Malformed { .. } | DecodeError::InvalidContent(_)) => {
            "Les données enregistrées sont illisibles."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_raw_export() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        assert_eq!(raw_export_file_name(today), "gym-brut-2026-11-02.json");
    }

    #[test]
    fn explains_each_kind_of_failure() {
        assert_eq!(
            load_failure_message(&LoadError::Storage(StorageError::Unavailable)),
            "Le stockage du navigateur est inaccessible."
        );
        assert_eq!(
            load_failure_message(&LoadError::Decode(DecodeError::UnsupportedSchemaVersion(2))),
            "Les données ont été enregistrées par une version plus récente de l'application."
        );
        assert_eq!(
            load_failure_message(&LoadError::Decode(DecodeError::Malformed {
                line: 1,
                column: 1
            })),
            "Les données enregistrées sont illisibles."
        );
        assert_eq!(
            load_failure_message(&LoadError::Storage(StorageError::CorruptedContent)),
            "Les données enregistrées sont illisibles."
        );
    }
}
