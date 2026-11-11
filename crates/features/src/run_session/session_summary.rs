use domain::rest::average_rest;
use domain::weekly::{done_count_in_week, week_start};
use domain::{Db, SessionId, SessionStatus};

use super::session_records::session_records;
use crate::shared::exercise_name::exercise_name;
use crate::shared::format::{
    format_clock, format_long_duration, format_set_measure, format_week_count,
};
use crate::shared::records::RecordBadge;
use crate::shared::session_volume::session_volume_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    pub name: String,
    pub duration: String,
    pub set_count: usize,
    pub volume: Option<String>,
    pub average_rest: Option<String>,
    pub records: Vec<RecordBadge>,
    pub week_text: String,
    pub exercises: Vec<SummaryExercise>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryExercise {
    pub name: String,
    pub sets: Vec<String>,
}

// End-of-session screen.
pub fn session_summary(db: &Db, session_id: SessionId) -> Option<SessionSummary> {
    let session = db.session(session_id)?;
    let SessionStatus::Done {
        started_at,
        finished_at,
    } = session.status()
    else {
        return None;
    };
    let week = week_start(started_at.date_naive());

    Some(SessionSummary {
        name: session.name().to_string(),
        duration: format_long_duration((*finished_at - *started_at).num_seconds()),
        set_count: session
            .entries()
            .iter()
            .map(|entry| entry.sets().len())
            .sum(),
        volume: session_volume_text(db, session),
        average_rest: average_rest(session).map(|rest| format_clock(rest.num_seconds())),
        records: session_records(db, session_id),
        week_text: format_week_count(
            done_count_in_week(db, week),
            db.preferences().weekly_goal().map(domain::WeeklyGoal::get),
        ),
        exercises: session
            .entries()
            .iter()
            .map(|entry| SummaryExercise {
                name: exercise_name(db, entry.exercise_id()),
                sets: entry
                    .sets()
                    .iter()
                    .map(|set| format_set_measure(set.measure()))
                    .collect(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use domain::EntryId;

    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, start_session, weight_reps};

    #[test]
    fn summarizes_a_done_session() {
        let mut db = Db::default();
        let id = done_session(
            &mut db,
            28,
            &[
                (bench(), vec![weight_reps(60, 10), weight_reps(80, 8)]),
                (pull_up(), vec![reps(10)]),
            ],
        );
        db.update_set(id, EntryId::new(1), 0, weight_reps(60, 10), true)
            .unwrap();

        let summary = session_summary(&db, id).unwrap();
        assert_eq!(summary.name, "Push A");
        assert_eq!(summary.duration, "1 h 00");
        assert_eq!(summary.set_count, 3);
        assert_eq!(summary.volume.as_deref(), Some("640 kg"));
        assert_eq!(summary.average_rest.as_deref(), Some("5:00"));
        assert!(summary.records.is_empty());
        assert_eq!(summary.week_text, "1 séance cette semaine");
        assert_eq!(
            summary.exercises,
            [
                SummaryExercise {
                    name: "Développé couché barre".to_owned(),
                    sets: vec!["60 × 10".to_owned(), "80 × 8".to_owned()],
                },
                SummaryExercise {
                    name: "Tractions".to_owned(),
                    sets: vec!["10 reps".to_owned()],
                },
            ]
        );
    }

    #[test]
    fn lists_broken_records_and_weekly_goal() {
        let mut db = Db::default();
        db.set_weekly_goal(domain::WeeklyGoal::new(3).ok());
        done_session(&mut db, 26, &[(bench(), vec![weight_reps(80, 8)])]);
        let id = done_session(&mut db, 28, &[(bench(), vec![weight_reps(82, 8)])]);
        let summary = session_summary(&db, id).unwrap();
        assert_eq!(summary.week_text, "2 / 3 séances cette semaine");
        let texts: Vec<&str> = summary
            .records
            .iter()
            .map(|badge| badge.text.as_str())
            .collect();
        assert_eq!(texts[0], "Poids max 82 kg (avant 80 kg)");
    }

    #[test]
    fn has_no_volume_without_weighted_sets_and_needs_a_done_session() {
        let mut db = Db::default();
        let id = done_session(&mut db, 28, &[(pull_up(), vec![reps(10)])]);
        assert_eq!(session_summary(&db, id).unwrap().volume, None);
        let current = start_session(&mut db, &[]);
        assert_eq!(session_summary(&db, current), None);
    }
}
