use domain::{Db, ExerciseId, SessionId, SessionStatus};

use crate::shared::format::{format_long_duration, format_set_count, format_short_date};
use crate::shared::session_volume::session_volume_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub id: SessionId,
    pub name: String,
    pub date_text: String,
    pub summary: String,
}

// Done sessions, most recent first; optionally only those where an exercise has sets.
pub fn history(db: &Db, exercise: Option<&ExerciseId>) -> Vec<HistoryItem> {
    let mut done: Vec<_> = db
        .sessions()
        .iter()
        .filter_map(|session| match session.status() {
            SessionStatus::Done {
                started_at,
                finished_at,
            } => Some((session, *started_at, *finished_at)),
            SessionStatus::Planned | SessionStatus::InProgress { .. } => None,
        })
        .filter(|(session, _, _)| {
            exercise.is_none_or(|exercise| {
                session
                    .entries()
                    .iter()
                    .any(|entry| entry.exercise_id() == exercise && !entry.sets().is_empty())
            })
        })
        .collect();
    done.sort_by_key(|(_, _, finished_at)| std::cmp::Reverse(*finished_at));
    done.into_iter()
        .map(|(session, started_at, finished_at)| {
            let exercise_count = session.entries().len();
            let set_count = session
                .entries()
                .iter()
                .map(|entry| entry.sets().len())
                .sum();
            HistoryItem {
                id: session.id(),
                name: session.name().to_string(),
                date_text: format_short_date(finished_at.date_naive()),
                summary: [
                    Some(format_long_duration(
                        (finished_at - started_at).num_seconds(),
                    )),
                    Some(exercise_count_text(exercise_count)),
                    Some(format_set_count(set_count)),
                    session_volume_text(db, session),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · "),
            }
        })
        .collect()
}

fn exercise_count_text(count: usize) -> String {
    match count {
        1 => "1 exercice".to_owned(),
        count => format!("{count} exercices"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, start_session, weight_reps};

    #[test]
    fn lists_done_sessions_most_recent_first() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(pull_up(), vec![reps(10)])]);
        done_session(
            &mut db,
            27,
            &[
                (bench(), vec![weight_reps(80, 8), weight_reps(80, 7)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        start_session(&mut db, &[]);

        let items = history(&db, None);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].date_text, "mar. 27 oct.");
        assert_eq!(
            items[0].summary,
            "1 h 00 · 2 exercices · 3 séries · 1\u{202f}200 kg"
        );
        assert_eq!(items[1].summary, "1 h 00 · 1 exercice · 1 série");
    }

    #[test]
    fn filters_by_exercise() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(pull_up(), vec![reps(10)])]);
        let with_bench = done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let items = history(&db, Some(&bench()));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, with_bench);
    }
}
