use domain::{BuiltinExercise, builtin_exercises};

// Pool exercises offered in lists: retired ones only remain readable in past sessions.
pub fn shown_builtin_exercises() -> impl Iterator<Item = &'static BuiltinExercise> {
    builtin_exercises()
        .iter()
        .filter(|exercise| !exercise.is_retired())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_retired_exercises() {
        let shown: Vec<&str> = shown_builtin_exercises().map(BuiltinExercise::id).collect();
        assert!(shown.contains(&"barbell_bench_press"));
        assert!(!shown.contains(&"foam_roll_quads"));
        assert!(shown.len() < builtin_exercises().len());
    }
}
