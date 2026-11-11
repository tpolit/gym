use domain::records::done_sessions_chronologically;
use domain::{Db, ExerciseId};

use crate::shared::exercise_name::exercise_name;
use crate::shared::format::format_short_date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticedExercise {
    pub id: ExerciseId,
    pub name: String,
    pub detail: String,
}

// Exercises with at least one set in a done session, the most recently practiced first.
pub fn practiced_exercises(db: &Db) -> Vec<PracticedExercise> {
    let mut practiced: Vec<(ExerciseId, usize, chrono::NaiveDate)> = Vec::new();
    for (session, started_at) in done_sessions_chronologically(db) {
        let mut seen_in_session: Vec<&ExerciseId> = Vec::new();
        for entry in session
            .entries()
            .iter()
            .filter(|entry| !entry.sets().is_empty())
        {
            if seen_in_session.contains(&entry.exercise_id()) {
                continue;
            }
            seen_in_session.push(entry.exercise_id());
            match practiced
                .iter_mut()
                .find(|(id, _, _)| id == entry.exercise_id())
            {
                Some((_, count, last)) => {
                    *count += 1;
                    *last = started_at.date_naive();
                }
                None => practiced.push((entry.exercise_id().clone(), 1, started_at.date_naive())),
            }
        }
    }
    practiced.sort_by_key(|(_, _, last)| std::cmp::Reverse(*last));
    practiced
        .into_iter()
        .map(|(id, count, last)| PracticedExercise {
            name: exercise_name(db, &id),
            detail: format!(
                "{count} séance{} · {}",
                if count > 1 { "s" } else { "" },
                format_short_date(last)
            ),
            id,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, weight_reps};

    #[test]
    fn lists_practiced_exercises_most_recent_first() {
        let mut db = Db::default();
        done_session(
            &mut db,
            20,
            &[
                (bench(), vec![weight_reps(80, 8)]),
                (bench(), vec![weight_reps(60, 8)]),
            ],
        );
        done_session(&mut db, 27, &[(pull_up(), vec![reps(10)])]);
        done_session(&mut db, 28, &[(pull_up(), vec![reps(10)])]);
        let exercises = practiced_exercises(&db);
        assert_eq!(exercises[0].id, pull_up());
        assert_eq!(exercises[0].detail, "2 séances · mer. 28 oct.");
        assert_eq!(exercises[1].detail, "1 séance · mar. 20 oct.");
    }
}
