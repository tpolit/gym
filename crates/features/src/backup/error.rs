use persistence::{DecodeError, EncodeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupError {
    Encode(EncodeError),
    Decode(DecodeError),
}

pub fn backup_error_message(error: &BackupError) -> &'static str {
    match error {
        BackupError::Encode(_) => "L'export a échoué.",
        BackupError::Decode(DecodeError::UnsupportedSchemaVersion(_)) => {
            "Ce fichier vient d'une version plus récente de l'application."
        }
        BackupError::Decode(DecodeError::Malformed { .. }) => {
            "Ce fichier n'est pas une sauvegarde Gym."
        }
        BackupError::Decode(DecodeError::InvalidContent(_)) => {
            "Cette sauvegarde contient des données incohérentes : rien n'a été modifié."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_in_french() {
        assert_eq!(
            backup_error_message(&BackupError::Decode(DecodeError::Malformed {
                line: 1,
                column: 1
            })),
            "Ce fichier n'est pas une sauvegarde Gym."
        );
    }
}
