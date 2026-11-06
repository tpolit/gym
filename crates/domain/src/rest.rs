// Rest before a set = its completion time − that of the set completed just before it
// in the session, across all exercises; unknown if either time is missing.
// It therefore includes the time spent performing the set.

use chrono::{DateTime, FixedOffset, TimeDelta};

use crate::session::{EntryId, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletedSet {
    pub entry_id: EntryId,
    pub index: usize,
    pub completed_at: DateTime<FixedOffset>,
}

pub fn latest_completed_set(session: &Session) -> Option<CompletedSet> {
    completed_sets(session).max_by_key(|set| set.completed_at)
}

// Indexed like the session: [entry index][set index].
pub fn rest_before_each_set(session: &Session) -> Vec<Vec<Option<TimeDelta>>> {
    let mut completion_times: Vec<_> = completed_sets(session)
        .map(|set| set.completed_at)
        .collect();
    completion_times.sort();
    session
        .entries()
        .iter()
        .map(|entry| {
            entry
                .sets()
                .iter()
                .map(|set| {
                    let completed_at = set.completed_at()?;
                    let earlier_count =
                        completion_times.partition_point(|time| *time < completed_at);
                    let previous = completion_times.get(earlier_count.checked_sub(1)?)?;
                    Some(completed_at - *previous)
                })
                .collect()
        })
        .collect()
}

pub fn average_rest(session: &Session) -> Option<TimeDelta> {
    let known_rests: Vec<TimeDelta> = rest_before_each_set(session)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    let count = i32::try_from(known_rests.len())
        .ok()
        .filter(|count| *count > 0)?;
    Some(known_rests.into_iter().sum::<TimeDelta>() / count)
}

fn completed_sets(session: &Session) -> impl Iterator<Item = CompletedSet> + '_ {
    session.entries().iter().flat_map(|entry| {
        entry
            .sets()
            .iter()
            .enumerate()
            .filter_map(move |(index, set)| {
                Some(CompletedSet {
                    entry_id: entry.entry_id(),
                    index,
                    completed_at: set.completed_at()?,
                })
            })
    })
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::TimeZone;

    use super::*;
    use crate::exercise::ExerciseId;
    use crate::measure::{SetMeasure, Weight};
    use crate::session::{SessionExercise, SessionId, SessionOrigin, SessionStatus};
    use crate::set::WorkoutSet;
    use crate::text::NonEmptyText;

    fn at(minute: u32, second: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, 28, 18, minute, second)
            .unwrap()
    }

    fn set(completed_at: Option<DateTime<FixedOffset>>) -> WorkoutSet {
        let measure = SetMeasure::WeightReps {
            weight: Weight::from_grams(80_000),
            reps: NonZeroU32::new(8).unwrap(),
        };
        WorkoutSet::new(measure, false, completed_at)
    }

    fn session(entries: Vec<Vec<WorkoutSet>>) -> Session {
        let entries = entries
            .into_iter()
            .zip(1..)
            .map(|(sets, id)| {
                SessionExercise::restore(
                    EntryId::new(id),
                    ExerciseId::Builtin("barbell_bench_press".to_owned()),
                    sets,
                )
            })
            .collect();
        Session::restore(
            SessionId::new(1),
            NonEmptyText::new("Push A").unwrap(),
            SessionOrigin::Adhoc,
            None,
            SessionStatus::InProgress {
                started_at: at(0, 0),
            },
            None,
            entries,
        )
        .unwrap()
    }

    fn minutes(minutes: i64, seconds: i64) -> Option<TimeDelta> {
        Some(TimeDelta::minutes(minutes) + TimeDelta::seconds(seconds))
    }

    #[test]
    fn rest_spans_exercises_and_ignores_unknown_times() {
        let session = session(vec![
            vec![set(Some(at(5, 0))), set(Some(at(7, 30)))],
            vec![set(None), set(Some(at(10, 0)))],
        ]);
        assert_eq!(
            rest_before_each_set(&session),
            vec![vec![None, minutes(2, 30)], vec![None, minutes(2, 30)]]
        );
    }

    #[test]
    fn rest_follows_completion_order_not_list_order() {
        let session = session(vec![vec![set(Some(at(10, 0)))], vec![set(Some(at(4, 0)))]]);
        assert_eq!(
            rest_before_each_set(&session),
            vec![vec![minutes(6, 0)], vec![None]]
        );
    }

    #[test]
    fn average_rest_uses_known_rests_only() {
        let session = session(vec![vec![
            set(Some(at(5, 0))),
            set(Some(at(7, 0))),
            set(None),
            set(Some(at(10, 0))),
        ]]);
        assert_eq!(average_rest(&session), minutes(2, 30));
        assert_eq!(
            average_rest(&self::session(vec![vec![set(Some(at(5, 0)))]])),
            None
        );
    }

    #[test]
    fn latest_completed_set_is_the_most_recent_one() {
        let session = session(vec![
            vec![set(Some(at(9, 0)))],
            vec![set(Some(at(4, 0))), set(None)],
        ]);
        assert_eq!(
            latest_completed_set(&session),
            Some(CompletedSet {
                entry_id: EntryId::new(1),
                index: 0,
                completed_at: at(9, 0),
            })
        );
        assert_eq!(
            latest_completed_set(&self::session(vec![vec![set(None)]])),
            None
        );
    }
}
