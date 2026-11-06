use domain::{Db, ExerciseId, ExerciseSettings, NonEmptyText};

use super::error::CatalogError;

// Blank text removes the settings.
pub fn set_exercise_settings(db: &mut Db, id: ExerciseId, text: &str) -> Result<(), CatalogError> {
    let settings = NonEmptyText::optional(text).map(ExerciseSettings::new);
    Ok(db.set_exercise_settings(id, settings)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::bench;

    #[test]
    fn sets_then_clears_settings() {
        let mut db = Db::default();
        set_exercise_settings(&mut db, bench(), " Banc cran 2 ").unwrap();
        assert_eq!(
            db.exercise_settings(&bench()).map(|s| s.text().as_str()),
            Some("Banc cran 2")
        );
        set_exercise_settings(&mut db, bench(), "").unwrap();
        assert!(db.exercise_settings(&bench()).is_none());
    }
}
