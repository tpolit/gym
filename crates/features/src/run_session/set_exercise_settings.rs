use domain::{Db, ExerciseId, ExerciseSettings, NonEmptyText};

use super::error::RunSessionError;

// Same domain method as in the catalog; blank text removes the settings.
pub fn set_exercise_settings(
    db: &mut Db,
    id: ExerciseId,
    text: &str,
) -> Result<(), RunSessionError> {
    let settings = NonEmptyText::optional(text).map(ExerciseSettings::new);
    Ok(db.set_exercise_settings(id, settings)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::bench;

    #[test]
    fn updates_settings_during_the_session() {
        let mut db = Db::default();
        set_exercise_settings(&mut db, bench(), "Banc cran 2").unwrap();
        assert_eq!(
            db.exercise_settings(&bench()).map(|s| s.text().as_str()),
            Some("Banc cran 2")
        );
        set_exercise_settings(&mut db, bench(), "  ").unwrap();
        assert!(db.exercise_settings(&bench()).is_none());
    }
}
