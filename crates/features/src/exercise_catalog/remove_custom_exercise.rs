use domain::{Db, Removed};

use super::error::CatalogError;

pub fn remove_custom_exercise(db: &mut Db, id: u32) -> Result<Removed, CatalogError> {
    Ok(db.remove_custom_exercise(id)?)
}

#[cfg(test)]
mod tests {
    use domain::{MeasureKind, NonEmptyText};

    use super::*;
    use crate::test_support::custom_info;

    #[test]
    fn deletes_an_unused_exercise() {
        let mut db = Db::default();
        db.add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        assert_eq!(remove_custom_exercise(&mut db, 1), Ok(Removed::Deleted));
        assert!(db.custom_exercises().is_empty());
    }

    #[test]
    fn archives_a_used_exercise() {
        let mut db = Db::default();
        let id = db
            .add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        db.plan_session(NonEmptyText::new("Dos").unwrap(), None, &[id])
            .unwrap();
        assert_eq!(remove_custom_exercise(&mut db, 1), Ok(Removed::Archived));
        assert!(db.custom_exercise(1).unwrap().is_archived());
    }
}
