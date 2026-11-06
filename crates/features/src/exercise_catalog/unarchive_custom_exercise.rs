use domain::Db;

use super::error::CatalogError;

pub fn unarchive_custom_exercise(db: &mut Db, id: u32) -> Result<(), CatalogError> {
    Ok(db.unarchive_custom_exercise(id)?)
}

#[cfg(test)]
mod tests {
    use domain::{DbError, ExerciseId, MeasureKind, NonEmptyText};

    use super::*;
    use crate::test_support::custom_info;

    #[test]
    fn restores_an_archived_exercise() {
        let mut db = Db::default();
        let id = db
            .add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        db.plan_session(NonEmptyText::new("Dos").unwrap(), None, &[id])
            .unwrap();
        db.remove_custom_exercise(1).unwrap();
        unarchive_custom_exercise(&mut db, 1).unwrap();
        assert!(!db.custom_exercise(1).unwrap().is_archived());
        assert_eq!(
            unarchive_custom_exercise(&mut db, 2),
            Err(CatalogError::Domain(DbError::ExerciseNotFound(
                ExerciseId::Custom(2)
            )))
        );
    }
}
