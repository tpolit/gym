use domain::records::{RecordKind, done_sessions_chronologically, session_bests};
use domain::{Db, ExerciseId, MeasureKind};

use crate::shared::exercise_name::exercise_name;
use crate::shared::format::format_short_date;
use crate::shared::records::{format_record_value, record_label};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseProgress {
    pub exercise_name: String,
    pub metric_label: &'static str,
    pub headline: Option<String>,
    pub trend: Option<String>,
    pub points: Vec<ProgressPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressPoint {
    pub date_text: String,
    pub value: u64,
    pub value_text: String,
    pub is_record: bool,
}

// One point per done session, using the metric that best shows progress for the
// exercise's measure kind. The first point reaching the maximum is the record.
pub fn exercise_progress(db: &Db, exercise_id: &ExerciseId) -> Option<ExerciseProgress> {
    let measure = db.exercise(exercise_id)?.measure();
    let sessions = done_sessions_chronologically(db);
    let has_distance = sessions.iter().any(|(session, _)| {
        session_bests(session, exercise_id).contains_key(&RecordKind::LongestDistance)
    });
    let kind = match measure {
        MeasureKind::WeightReps => RecordKind::BestOneRepMax,
        MeasureKind::Reps => RecordKind::MostReps,
        MeasureKind::Duration => RecordKind::LongestDuration,
        MeasureKind::Cardio if has_distance => RecordKind::LongestDistance,
        MeasureKind::Cardio => RecordKind::LongestDuration,
    };
    let mut points: Vec<ProgressPoint> = sessions
        .iter()
        .filter_map(|(session, started_at)| {
            let value = *session_bests(session, exercise_id).get(&kind)?;
            Some(ProgressPoint {
                date_text: format_short_date(started_at.date_naive()),
                value,
                value_text: format_record_value(kind, value),
                is_record: false,
            })
        })
        .collect();
    let best = points.iter().map(|point| point.value).max();
    if let Some(record) = points.iter_mut().find(|point| Some(point.value) == best) {
        record.is_record = true;
    }
    let trend = match (points.first(), points.last()) {
        (Some(first), Some(last)) if points.len() > 1 => Some(format!(
            "{} depuis le {}",
            format_difference(kind, first.value, last.value),
            first.date_text
        )),
        _ => None,
    };
    Some(ExerciseProgress {
        exercise_name: exercise_name(db, exercise_id),
        metric_label: record_label(kind),
        headline: points.last().map(|point| point.value_text.clone()),
        trend,
        points,
    })
}

fn format_difference(kind: RecordKind, from: u64, to: u64) -> String {
    match to.cmp(&from) {
        std::cmp::Ordering::Equal => "Stable".to_owned(),
        std::cmp::Ordering::Greater => format!("+{}", format_record_value(kind, to - from)),
        std::cmp::Ordering::Less => format!("−{}", format_record_value(kind, from - to)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{bench, done_session, pull_up, reps, weight_reps};

    #[test]
    fn follows_the_estimated_one_rep_max() {
        let mut db = Db::default();
        done_session(&mut db, 13, &[(bench(), vec![weight_reps(60, 1)])]);
        done_session(&mut db, 20, &[(bench(), vec![weight_reps(80, 1)])]);
        done_session(&mut db, 27, &[(bench(), vec![weight_reps(70, 1)])]);
        let progress = exercise_progress(&db, &bench()).unwrap();
        assert_eq!(progress.exercise_name, "Développé couché barre");
        assert_eq!(progress.metric_label, "1RM estimé");
        let values: Vec<u64> = progress.points.iter().map(|point| point.value).collect();
        assert_eq!(values, [60_000, 80_000, 70_000]);
        let records: Vec<bool> = progress
            .points
            .iter()
            .map(|point| point.is_record)
            .collect();
        assert_eq!(records, [false, true, false]);
        assert_eq!(progress.headline.as_deref(), Some("70 kg"));
        assert_eq!(
            progress.trend.as_deref(),
            Some("+10 kg depuis le mar. 13 oct.")
        );
    }

    #[test]
    fn uses_reps_for_bodyweight_exercises() {
        let mut db = Db::default();
        done_session(&mut db, 20, &[(pull_up(), vec![reps(10), reps(8)])]);
        let progress = exercise_progress(&db, &pull_up()).unwrap();
        assert_eq!(progress.metric_label, "Reps max");
        assert_eq!(progress.headline.as_deref(), Some("10 reps"));
        assert_eq!(progress.trend, None);
        assert_eq!(exercise_progress(&db, &ExerciseId::Custom(1)), None);
    }
}
