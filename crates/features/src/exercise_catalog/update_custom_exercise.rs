use domain::Db;

use super::error::CatalogError;
use super::exercise_form::{ExerciseForm, ensure_unique_name};

pub fn update_custom_exercise(
    db: &mut Db,
    id: u32,
    form: &ExerciseForm,
) -> Result<(), CatalogError> {
    let info = form.to_info()?;
    ensure_unique_name(db, &info, Some(id))?;
    Ok(db.update_custom_exercise(id, info)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, MeasureKind, NonEmptyText};

    use super::*;
    use crate::test_support::custom_info;

    #[test]
    fn renames_and_refuses_a_measure_change_once_used() {
        let mut db = Db::default();
        let id = db
            .add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        let mut form = ExerciseForm::of_custom_exercise(&db, 1).unwrap();
        form.name = "Rowing Technogym".to_owned();
        update_custom_exercise(&mut db, 1, &form).unwrap();
        assert_eq!(
            db.custom_exercise(1).map(|e| e.info().name().as_str()),
            Some("Rowing Technogym")
        );

        db.plan_session(NonEmptyText::new("Dos").unwrap(), None, &[id])
            .unwrap();
        form.measure = MeasureKind::Reps;
        assert_eq!(
            update_custom_exercise(&mut db, 1, &form),
            Err(CatalogError::Domain(DbError::MeasureLockedByUse))
        );
    }
}
