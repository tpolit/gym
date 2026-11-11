use chrono::TimeDelta;
use domain::rest::rest_before_each_set;
use domain::{Db, SessionStatus};

use crate::shared::format::{format_clock, format_long_duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAverages {
    pub session_count: usize,
    pub duration: String,
    pub sets: String,
    pub rest: Option<String>,
}

// Averages over every done session.
pub fn session_averages(db: &Db) -> Option<SessionAverages> {
    let done: Vec<_> = db
        .sessions()
        .iter()
        .filter_map(|session| match session.status() {
            SessionStatus::Done {
                started_at,
                finished_at,
            } => Some((session, *finished_at - *started_at)),
            SessionStatus::Planned | SessionStatus::InProgress { .. } => None,
        })
        .collect();
    let count = i64::try_from(done.len()).ok().filter(|count| *count > 0)?;
    let total_duration: TimeDelta = done.iter().map(|(_, duration)| *duration).sum();
    let total_sets: usize = done
        .iter()
        .flat_map(|(session, _)| session.entries())
        .map(|entry| entry.sets().len())
        .sum();
    let rests: Vec<TimeDelta> = done
        .iter()
        .flat_map(|(session, _)| rest_before_each_set(session))
        .flatten()
        .flatten()
        .collect();
    let rest_count = i64::try_from(rests.len()).unwrap_or(0);
    let average_tenths = i64::try_from(total_sets).unwrap_or(0) * 10 / count;
    Some(SessionAverages {
        session_count: done.len(),
        duration: format_long_duration(total_duration.num_seconds() / count),
        sets: if average_tenths % 10 == 0 {
            (average_tenths / 10).to_string()
        } else {
            format!("{},{}", average_tenths / 10, average_tenths % 10)
        },
        rest: (rest_count > 0).then(|| {
            let total: i64 = rests.iter().map(TimeDelta::num_seconds).sum();
            format_clock(total / rest_count)
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, weight_reps};

    #[test]
    fn averages_duration_sets_and_rest() {
        let mut db = Db::default();
        assert_eq!(session_averages(&db), None);
        done_session(
            &mut db,
            20,
            &[(bench(), vec![weight_reps(80, 8), weight_reps(80, 8)])],
        );
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(80, 8)])]);
        let averages = session_averages(&db).unwrap();
        assert_eq!(averages.session_count, 2);
        assert_eq!(averages.duration, "1 h 00");
        assert_eq!(averages.sets, "1,5");
        assert_eq!(averages.rest.as_deref(), Some("5:00"));
    }
}
