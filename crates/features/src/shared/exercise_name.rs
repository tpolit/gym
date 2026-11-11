use domain::{Db, ExerciseId};

pub fn exercise_name(db: &Db, id: &ExerciseId) -> String {
    db.exercise(id)
        .map(|info| info.name().to_string())
        .unwrap_or_else(|| "Exercice inconnu".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::bench;

    #[test]
    fn resolves_builtin_and_unknown_exercises() {
        let db = Db::default();
        assert_eq!(exercise_name(&db, &bench()), "Développé couché barre");
        assert_eq!(
            exercise_name(&db, &ExerciseId::Custom(9)),
            "Exercice inconnu"
        );
    }
}
