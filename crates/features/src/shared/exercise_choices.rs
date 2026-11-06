use std::collections::BTreeSet;

use domain::{Db, ExerciseId, ExerciseInfo, Session, SessionStatus};

use crate::shared::exercise_choice::{ExerciseChoice, ExerciseChoices};
use crate::shared::labels::{equipment_label, muscle_group_label};
use crate::shared::search::normalize_for_search;
use crate::shared::shown_builtin_exercises::shown_builtin_exercises;

const RECENT_LIMIT: usize = 8;

// Exercise picker: recent ones first, then every non-archived exercise.
pub fn exercise_choices(db: &Db) -> ExerciseChoices {
    let builtin = shown_builtin_exercises().map(|exercise| {
        let mut search_terms = vec![exercise.info().name().to_string()];
        search_terms.extend(exercise.aliases().iter().map(|alias| (*alias).to_owned()));
        choice(exercise.exercise_id(), exercise.info(), search_terms)
    });
    let custom = db
        .custom_exercises()
        .iter()
        .filter(|exercise| !exercise.is_archived())
        .map(|exercise| {
            choice(
                exercise.exercise_id(),
                exercise.info(),
                vec![exercise.info().name().to_string()],
            )
        });
    let mut all: Vec<ExerciseChoice> = builtin.chain(custom).collect();
    all.sort_by_cached_key(|choice| normalize_for_search(&choice.name));

    let recent = recent_exercise_ids(db)
        .into_iter()
        .filter_map(|id| all.iter().find(|choice| choice.id == id).cloned())
        .take(RECENT_LIMIT)
        .collect();
    ExerciseChoices { recent, all }
}

fn recent_exercise_ids(db: &Db) -> Vec<ExerciseId> {
    let mut started: Vec<&Session> = db
        .sessions()
        .iter()
        .filter(|session| !matches!(session.status(), SessionStatus::Planned))
        .collect();
    started.sort_by_key(|session| std::cmp::Reverse(session.status().started_at()));
    let mut seen = BTreeSet::new();
    started
        .into_iter()
        .flat_map(Session::entries)
        .map(|entry| entry.exercise_id().clone())
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

fn choice(id: ExerciseId, info: &ExerciseInfo, search_terms: Vec<String>) -> ExerciseChoice {
    ExerciseChoice {
        id,
        name: info.name().to_string(),
        detail: format!(
            "{} · {}",
            muscle_group_label(info.muscle_group()),
            equipment_label(info.equipment())
        ),
        search_terms,
    }
}

#[cfg(test)]
mod tests {
    use domain::{Category, Equipment, MeasureKind, MuscleGroup, NonEmptyText};

    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, start_session, weight_reps};

    fn custom_info(name: &str) -> ExerciseInfo {
        ExerciseInfo::new(
            NonEmptyText::new(name).unwrap(),
            Category::Strength,
            MuscleGroup::Back,
            Equipment::Machine,
            MeasureKind::WeightReps,
        )
        .unwrap()
    }

    #[test]
    fn lists_every_exercise_sorted_without_archived_ones() {
        let mut db = Db::default();
        db.add_custom_exercise(custom_info("Archivé")).unwrap();
        db.remove_custom_exercise(1).unwrap();
        let rowing = db
            .add_custom_exercise(custom_info("Rowing machine"))
            .unwrap();

        let choices = exercise_choices(&db);
        assert_eq!(choices.all.len(), shown_builtin_exercises().count() + 1);
        assert!(
            !choices
                .all
                .iter()
                .any(|choice| choice.id == ExerciseId::Builtin("foam_roll_quads".to_owned()))
        );
        assert!(choices.all.iter().any(|choice| choice.id == rowing));
        assert!(!choices.all.iter().any(|choice| choice.name == "Archivé"));
        let names: Vec<String> = choices
            .all
            .iter()
            .map(|c| normalize_for_search(&c.name))
            .collect();
        assert!(names.is_sorted());

        let bench_choice = choices
            .all
            .iter()
            .find(|choice| choice.id == bench())
            .unwrap();
        assert_eq!(bench_choice.detail, "Pectoraux · Barre");
        assert!(bench_choice.matches("DC"));
    }

    #[test]
    fn recent_exercises_come_from_the_latest_sessions_first() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 8)])]);
        done_session(
            &mut db,
            27,
            &[
                (pull_up(), vec![reps(10)]),
                (bench(), vec![weight_reps(80, 8)]),
            ],
        );
        start_session(&mut db, &[]);

        let recent: Vec<ExerciseId> = exercise_choices(&db)
            .recent
            .into_iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(recent, [pull_up(), bench()]);
    }
}
