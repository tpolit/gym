use std::collections::BTreeSet;

use domain::{Db, ExerciseId, SessionStatus};

use crate::shared::exercise_name::exercise_name;
use crate::shared::search::normalize_for_search;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryExerciseChoice {
    pub id: ExerciseId,
    pub name: String,
}

// Exercises available in the history filter: those with sets in a done session.
pub fn history_exercise_choices(db: &Db) -> Vec<HistoryExerciseChoice> {
    let practiced: BTreeSet<&ExerciseId> = db
        .sessions()
        .iter()
        .filter(|session| matches!(session.status(), SessionStatus::Done { .. }))
        .flat_map(|session| session.entries())
        .filter(|entry| !entry.sets().is_empty())
        .map(|entry| entry.exercise_id())
        .collect();
    let mut choices: Vec<HistoryExerciseChoice> = practiced
        .into_iter()
        .map(|id| HistoryExerciseChoice {
            id: id.clone(),
            name: exercise_name(db, id),
        })
        .collect();
    choices.sort_by_cached_key(|choice| normalize_for_search(&choice.name));
    choices
}

pub fn history_filter_title(db: &Db, exercise: Option<&ExerciseId>) -> String {
    exercise.map_or_else(
        || "Toutes les séances".to_owned(),
        |id| format!("Séances avec {}", exercise_name(db, id)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, start_session, weight_reps};

    #[test]
    fn lists_practiced_exercises_by_name() {
        let mut db = Db::default();
        done_session(
            &mut db,
            20,
            &[
                (pull_up(), vec![reps(10)]),
                (bench(), vec![weight_reps(80, 8)]),
            ],
        );
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let in_progress = start_session(&mut db, &[ExerciseId::Builtin("plank".to_owned())]);
        db.add_entry(in_progress, ExerciseId::Builtin("leg_press".to_owned()))
            .unwrap();

        let names: Vec<String> = history_exercise_choices(&db)
            .into_iter()
            .map(|choice| choice.name)
            .collect();
        assert_eq!(names, ["Développé couché barre", "Tractions"]);
    }

    #[test]
    fn titles_the_filtered_list() {
        let db = Db::default();
        assert_eq!(history_filter_title(&db, None), "Toutes les séances");
        assert_eq!(
            history_filter_title(&db, Some(&bench())),
            "Séances avec Développé couché barre"
        );
    }
}
