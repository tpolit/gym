use domain::{DbError, ExerciseInfoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    EmptyName,
    DuplicateName,
    MeasureNotAllowed(ExerciseInfoError),
    Domain(DbError),
}

impl From<DbError> for CatalogError {
    fn from(error: DbError) -> Self {
        Self::Domain(error)
    }
}

pub fn catalog_error_message(error: &CatalogError) -> &'static str {
    match error {
        CatalogError::EmptyName => "Donnez un nom à l'exercice.",
        CatalogError::DuplicateName => "Un exercice porte déjà ce nom.",
        CatalogError::MeasureNotAllowed(_) => {
            "Ce type de mesure n'est pas possible pour cette catégorie."
        }
        CatalogError::Domain(DbError::MeasureLockedByUse) => {
            "Le type de mesure ne peut plus changer : l'exercice est déjà utilisé."
        }
        CatalogError::Domain(_) => "Action impossible.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_in_french() {
        assert_eq!(
            catalog_error_message(&CatalogError::DuplicateName),
            "Un exercice porte déjà ce nom."
        );
        assert_eq!(
            catalog_error_message(&CatalogError::Domain(DbError::IdOverflow)),
            "Action impossible."
        );
    }
}
