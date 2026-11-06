use domain::{Db, ExerciseId};

use super::error::CatalogError;
use super::exercise_form::{ExerciseForm, ensure_unique_name};

pub fn create_custom_exercise(
    db: &mut Db,
    form: &ExerciseForm,
) -> Result<ExerciseId, CatalogError> {
    let info = form.to_info()?;
    ensure_unique_name(db, &info, None)?;
    Ok(db.add_custom_exercise(info)?)
}

#[cfg(test)]
mod tests {
    use domain::{Category, MeasureKind};

    use super::*;

    fn form(name: &str) -> ExerciseForm {
        ExerciseForm {
            name: name.to_owned(),
            ..ExerciseForm::blank()
        }
    }

    #[test]
    fn creates_the_exercise() {
        let mut db = Db::default();
        let id = create_custom_exercise(&mut db, &form("Rowing machine")).unwrap();
        assert_eq!(id, ExerciseId::Custom(1));
        assert_eq!(
            db.exercise(&id).map(|info| info.name().as_str()),
            Some("Rowing machine")
        );
    }

    #[test]
    fn refuses_invalid_or_duplicate_exercises() {
        let mut db = Db::default();
        assert_eq!(
            create_custom_exercise(&mut db, &form("  ")),
            Err(CatalogError::EmptyName)
        );
        assert_eq!(
            create_custom_exercise(&mut db, &form("Tractions")),
            Err(CatalogError::DuplicateName)
        );
        let forbidden = ExerciseForm {
            category: Category::Cardio,
            measure: MeasureKind::Reps,
            ..form("Vélo elliptique")
        };
        assert!(matches!(
            create_custom_exercise(&mut db, &forbidden),
            Err(CatalogError::MeasureNotAllowed(_))
        ));
        assert!(db.custom_exercises().is_empty());
    }
}
