// Personal records, shared by the session in progress, the end-of-session screen and the stats
// Warm-up sets never count. Values are integers in base units:
// grams, grams × reps, reps, seconds, meters, seconds per kilometer.

use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};

use crate::db::Db;
use crate::exercise::ExerciseId;
use crate::measure::SetMeasure;
use crate::session::{Session, SessionId, SessionStatus};
use crate::set::WorkoutSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecordKind {
    HeaviestWeight,
    BestOneRepMax,
    BestSessionVolume,
    MostReps,
    HeaviestLoad,
    LongestDuration,
    LongestDistance,
    BestPace,
}

impl RecordKind {
    pub const fn is_lower_better(self) -> bool {
        matches!(self, Self::BestPace)
    }

    pub const fn beats(self, candidate: u64, reference: u64) -> bool {
        if self.is_lower_better() {
            candidate < reference
        } else {
            candidate > reference
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonalRecord {
    pub kind: RecordKind,
    pub value: u64,
    pub session_id: SessionId,
    pub achieved_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenRecord {
    pub exercise_id: ExerciseId,
    pub kind: RecordKind,
    pub value: u64,
    pub previous: u64,
}

// Epley formula: weight × (1 + reps / 30); a single rep is the weight itself.
pub fn estimated_one_rep_max(weight_grams: u32, reps: u32) -> u64 {
    if reps <= 1 {
        u64::from(weight_grams)
    } else {
        u64::from(weight_grams) * u64::from(30 + reps) / 30
    }
}

pub fn weight_volume_grams<'a>(sets: impl IntoIterator<Item = &'a WorkoutSet>) -> u64 {
    sets.into_iter()
        .filter(|set| !set.is_warmup())
        .filter_map(|set| match set.measure() {
            SetMeasure::WeightReps { weight, reps } => {
                Some(u64::from(weight.grams()) * u64::from(reps.get()))
            }
            SetMeasure::Reps { .. } | SetMeasure::Duration { .. } | SetMeasure::Cardio(_) => None,
        })
        .sum()
}

pub fn exercise_sets<'a>(
    session: &'a Session,
    exercise_id: &'a ExerciseId,
) -> impl Iterator<Item = &'a WorkoutSet> + 'a {
    session
        .entries()
        .iter()
        .filter(move |entry| entry.exercise_id() == exercise_id)
        .flat_map(|entry| entry.sets())
}

// Best value of each record kind for one exercise within one session.
pub fn session_bests(session: &Session, exercise_id: &ExerciseId) -> BTreeMap<RecordKind, u64> {
    let mut bests = BTreeMap::new();
    let mut keep = |kind: RecordKind, value: u64| {
        bests
            .entry(kind)
            .and_modify(|best: &mut u64| {
                if kind.beats(value, *best) {
                    *best = value;
                }
            })
            .or_insert(value);
    };
    for set in exercise_sets(session, exercise_id).filter(|set| !set.is_warmup()) {
        match set.measure() {
            SetMeasure::WeightReps { weight, reps } => {
                keep(RecordKind::HeaviestWeight, u64::from(weight.grams()));
                keep(
                    RecordKind::BestOneRepMax,
                    estimated_one_rep_max(weight.grams(), reps.get()),
                );
            }
            SetMeasure::Reps { reps, load } => {
                keep(RecordKind::MostReps, u64::from(reps.get()));
                if let Some(load) = load {
                    keep(RecordKind::HeaviestLoad, u64::from(load.grams()));
                }
            }
            SetMeasure::Duration { duration, load } => {
                keep(RecordKind::LongestDuration, u64::from(duration.get()));
                if let Some(load) = load {
                    keep(RecordKind::HeaviestLoad, u64::from(load.grams()));
                }
            }
            SetMeasure::Cardio(cardio) => {
                keep(
                    RecordKind::LongestDuration,
                    u64::from(cardio.duration().get()),
                );
                if let Some(distance) = cardio.distance() {
                    keep(RecordKind::LongestDistance, u64::from(distance.get()));
                    keep(
                        RecordKind::BestPace,
                        u64::from(cardio.duration().get()) * 1_000 / u64::from(distance.get()),
                    );
                }
            }
        }
    }
    let volume = weight_volume_grams(exercise_sets(session, exercise_id));
    if volume > 0 {
        bests.insert(RecordKind::BestSessionVolume, volume);
    }
    bests
}

// Done sessions in chronological order, with their start time.
pub fn done_sessions_chronologically(db: &Db) -> Vec<(&Session, DateTime<FixedOffset>)> {
    let mut done: Vec<_> = db
        .sessions()
        .iter()
        .filter_map(|session| match session.status() {
            SessionStatus::Done { started_at, .. } => Some((session, *started_at)),
            SessionStatus::Planned | SessionStatus::InProgress { .. } => None,
        })
        .collect();
    done.sort_by_key(|(session, started_at)| (*started_at, session.id()));
    done
}

// The first session reaching the best value holds the record.
pub fn personal_records(db: &Db, exercise_id: &ExerciseId) -> Vec<PersonalRecord> {
    let mut records: BTreeMap<RecordKind, PersonalRecord> = BTreeMap::new();
    for (session, started_at) in done_sessions_chronologically(db) {
        for (kind, value) in session_bests(session, exercise_id) {
            let improves = records
                .get(&kind)
                .is_none_or(|record| kind.beats(value, record.value));
            if improves {
                records.insert(
                    kind,
                    PersonalRecord {
                        kind,
                        value,
                        session_id: session.id(),
                        achieved_at: started_at,
                    },
                );
            }
        }
    }
    records.into_values().collect()
}

// A record is broken only when an earlier done session had a value to beat:
// doing an exercise for the first time is not a record.
pub fn records_broken_in(db: &Db, session_id: SessionId) -> Vec<BrokenRecord> {
    let Some(session) = db.session(session_id) else {
        return Vec::new();
    };
    let Some(started_at) = session.status().started_at() else {
        return Vec::new();
    };
    let earlier: Vec<&Session> = done_sessions_chronologically(db)
        .into_iter()
        .filter(|(other, other_start)| other.id() != session_id && *other_start < started_at)
        .map(|(other, _)| other)
        .collect();
    let mut exercises: Vec<&ExerciseId> = Vec::new();
    for entry in session.entries() {
        if !exercises.contains(&entry.exercise_id()) {
            exercises.push(entry.exercise_id());
        }
    }
    let mut broken = Vec::new();
    for exercise_id in exercises {
        let mut previous_bests: BTreeMap<RecordKind, u64> = BTreeMap::new();
        for other in &earlier {
            for (kind, value) in session_bests(other, exercise_id) {
                previous_bests
                    .entry(kind)
                    .and_modify(|best| {
                        if kind.beats(value, *best) {
                            *best = value;
                        }
                    })
                    .or_insert(value);
            }
        }
        for (kind, value) in session_bests(session, exercise_id) {
            if let Some(previous) = previous_bests.get(&kind)
                && kind.beats(value, *previous)
            {
                broken.push(BrokenRecord {
                    exercise_id: exercise_id.clone(),
                    kind,
                    value,
                    previous: *previous,
                });
            }
        }
    }
    broken
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::TimeZone;

    use super::*;
    use crate::measure::{CardioMeasure, Load, Meters, Seconds, Weight};
    use crate::session::EntryId;
    use crate::text::NonEmptyText;

    fn at(day: u32, hour: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(3_600)
            .unwrap()
            .with_ymd_and_hms(2026, 10, day, hour, 0, 0)
            .unwrap()
    }

    fn bench() -> ExerciseId {
        ExerciseId::Builtin("barbell_bench_press".to_owned())
    }

    fn weight_reps(kg: u32, reps: u32) -> SetMeasure {
        SetMeasure::WeightReps {
            weight: Weight::from_grams(kg * 1_000),
            reps: NonZeroU32::new(reps).unwrap(),
        }
    }

    fn session_with(
        db: &mut Db,
        day: u32,
        exercise: ExerciseId,
        sets: &[(SetMeasure, bool)],
    ) -> SessionId {
        let id = db
            .start_new_session(NonEmptyText::new("Push").unwrap(), &[exercise], at(day, 18))
            .unwrap();
        for (measure, warmup) in sets {
            db.add_set(
                id,
                EntryId::new(1),
                WorkoutSet::new(*measure, *warmup, None),
            )
            .unwrap();
        }
        db.finish_session(id, at(day, 19)).unwrap();
        id
    }

    #[test]
    fn epley_estimate() {
        assert_eq!(estimated_one_rep_max(80_000, 8), 101_333);
        assert_eq!(estimated_one_rep_max(100_000, 1), 100_000);
    }

    #[test]
    fn session_bests_ignore_warmups() {
        let mut db = Db::default();
        let id = session_with(
            &mut db,
            20,
            bench(),
            &[
                (weight_reps(100, 3), true),
                (weight_reps(80, 8), false),
                (weight_reps(85, 5), false),
            ],
        );
        let bests = session_bests(db.session(id).unwrap(), &bench());
        assert_eq!(bests[&RecordKind::HeaviestWeight], 85_000);
        assert_eq!(bests[&RecordKind::BestOneRepMax], 101_333);
        assert_eq!(
            bests[&RecordKind::BestSessionVolume],
            80_000 * 8 + 85_000 * 5
        );
    }

    #[test]
    fn cardio_and_loaded_sets_have_their_own_records() {
        let mut db = Db::default();
        let rower = ExerciseId::Builtin("indoor_rower".to_owned());
        let cardio = SetMeasure::Cardio(CardioMeasure::new(
            Seconds::new(1_200).unwrap(),
            Meters::new(5_000).ok(),
            None,
            None,
            None,
        ));
        let id = session_with(&mut db, 20, rower.clone(), &[(cardio, false)]);
        let bests = session_bests(db.session(id).unwrap(), &rower);
        assert_eq!(bests[&RecordKind::LongestDistance], 5_000);
        assert_eq!(bests[&RecordKind::BestPace], 240);

        let pull_up = ExerciseId::Builtin("pull_up".to_owned());
        let loaded = SetMeasure::Reps {
            reps: NonZeroU32::new(6).unwrap(),
            load: Load::from_grams(10_000).ok(),
        };
        let id = session_with(&mut db, 21, pull_up.clone(), &[(loaded, false)]);
        let bests = session_bests(db.session(id).unwrap(), &pull_up);
        assert_eq!(bests[&RecordKind::MostReps], 6);
        assert_eq!(bests[&RecordKind::HeaviestLoad], 10_000);
    }

    #[test]
    fn personal_records_keep_the_first_session_reaching_the_best() {
        let mut db = Db::default();
        let first = session_with(&mut db, 20, bench(), &[(weight_reps(80, 8), false)]);
        let second = session_with(
            &mut db,
            27,
            bench(),
            &[(weight_reps(80, 8), false), (weight_reps(70, 12), false)],
        );
        let records = personal_records(&db, &bench());
        let heaviest = records
            .iter()
            .find(|r| r.kind == RecordKind::HeaviestWeight)
            .unwrap();
        assert_eq!(heaviest.session_id, first);
        let volume = records
            .iter()
            .find(|r| r.kind == RecordKind::BestSessionVolume)
            .unwrap();
        assert_eq!(volume.session_id, second);
    }

    #[test]
    fn broken_records_need_an_earlier_value() {
        let mut db = Db::default();
        let first = session_with(&mut db, 20, bench(), &[(weight_reps(80, 8), false)]);
        assert!(records_broken_in(&db, first).is_empty());
        let second = session_with(&mut db, 27, bench(), &[(weight_reps(85, 5), false)]);
        let broken = records_broken_in(&db, second);
        assert_eq!(
            broken,
            [BrokenRecord {
                exercise_id: bench(),
                kind: RecordKind::HeaviestWeight,
                value: 85_000,
                previous: 80_000,
            }]
        );
    }

    #[test]
    fn lower_pace_is_better() {
        assert!(RecordKind::BestPace.beats(230, 240));
        assert!(!RecordKind::HeaviestWeight.beats(230, 240));
    }
}
