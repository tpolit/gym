use chrono::NaiveDate;
use domain::{Db, ExerciseId, SessionId, SessionStatus, WorkoutSet};

use crate::shared::format::{format_set_measure, format_short_date};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastPerformance {
    pub date: NaiveDate,
    pub sets: Vec<WorkoutSet>,
}

// Most recent done session, excluding the current one, last occurrence of the exercise
// with at least one set. Read once by the view, without subscribing: it stays frozen.
pub fn last_performance(
    db: &Db,
    exercise_id: &ExerciseId,
    current_session: SessionId,
) -> Option<LastPerformance> {
    db.sessions()
        .iter()
        .filter(|session| session.id() != current_session)
        .filter_map(|session| match session.status() {
            SessionStatus::Done { finished_at, .. } => Some((*finished_at, session)),
            SessionStatus::Planned | SessionStatus::InProgress { .. } => None,
        })
        .filter_map(|(finished_at, session)| {
            session
                .entries()
                .iter()
                .rev()
                .find(|entry| entry.exercise_id() == exercise_id && !entry.sets().is_empty())
                .map(|entry| (finished_at, entry))
        })
        .max_by_key(|(finished_at, _)| *finished_at)
        .map(|(finished_at, entry)| LastPerformance {
            date: finished_at.date_naive(),
            sets: entry.sets().to_vec(),
        })
}

pub fn last_performance_text(last: &LastPerformance) -> String {
    let sets: Vec<String> = last
        .sets
        .iter()
        .map(|set| format_set_measure(set.measure()))
        .collect();
    format!("{} · {}", format_short_date(last.date), sets.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        add_set_at, bench, done_session, pull_up, reps, start_session, weight_reps,
    };

    fn measures(last: &LastPerformance) -> Vec<domain::SetMeasure> {
        last.sets.iter().map(|set| *set.measure()).collect()
    }

    #[test]
    fn takes_the_most_recent_done_session_and_last_occurrence() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(70, 10)])]);
        done_session(
            &mut db,
            27,
            &[
                (bench(), vec![weight_reps(80, 8), weight_reps(80, 7)]),
                (pull_up(), vec![reps(10)]),
                (bench(), vec![weight_reps(60, 15)]),
            ],
        );
        let current = start_session(&mut db, &[bench()]);
        add_set_at(&mut db, current, 1, weight_reps(90, 5), 5);

        let last = last_performance(&db, &bench(), current).unwrap();
        assert_eq!(last.date, NaiveDate::from_ymd_opt(2026, 10, 27).unwrap());
        assert_eq!(measures(&last), [weight_reps(60, 15)]);
        assert_eq!(last_performance_text(&last), "mar. 27 oct. · 60 × 15");
    }

    #[test]
    fn ignores_the_current_session_and_never_practiced_exercises() {
        let mut db = Db::default();
        let current = start_session(&mut db, &[bench()]);
        add_set_at(&mut db, current, 1, weight_reps(90, 5), 5);
        assert_eq!(last_performance(&db, &bench(), current), None);
        assert_eq!(last_performance(&db, &pull_up(), current), None);
    }

    #[test]
    fn lists_every_set_of_the_occurrence() {
        let mut db = Db::default();
        done_session(
            &mut db,
            20,
            &[(bench(), vec![weight_reps(60, 10), weight_reps(80, 8)])],
        );
        let current = start_session(&mut db, &[]);
        let last = last_performance(&db, &bench(), current).unwrap();
        assert_eq!(
            last_performance_text(&last),
            "mar. 20 oct. · 60 × 10, 80 × 8"
        );
    }
}
