// Volume of a session, warm-ups excluded: weight × reps for weighted sets, and
// (body weight + load) × reps for bodyweight strength exercises, using the body weight in
// effect on the session date. Those sets only count once a body weight is known.

use crate::db::Db;
use crate::exercise::{Category, Equipment};
use crate::measure::SetMeasure;
use crate::session::Session;

// None when no set counts: "0 kg" would be misleading for cardio or stretching.
pub fn session_volume_grams(db: &Db, session: &Session) -> Option<u64> {
    let body_weight = session
        .status()
        .started_at()
        .and_then(|started_at| db.preferences().body_weights().on(started_at.date_naive()))
        .or_else(|| {
            db.preferences()
                .body_weights()
                .latest()
                .map(|(_, weight)| weight)
        });
    let set_volumes: Vec<u64> = session
        .entries()
        .iter()
        .flat_map(|entry| {
            let lifts_body_weight = db.exercise(entry.exercise_id()).is_some_and(|info| {
                info.category() == Category::Strength && info.equipment() == Equipment::Bodyweight
            });
            entry
                .sets()
                .iter()
                .filter(|set| !set.is_warmup())
                .filter_map(move |set| match set.measure() {
                    SetMeasure::WeightReps { weight, reps } => Some((weight.grams(), reps.get())),
                    SetMeasure::Reps { reps, load } if lifts_body_weight => {
                        body_weight.map(|body| {
                            (
                                body.grams() + load.map_or(0, |load| load.grams()),
                                reps.get(),
                            )
                        })
                    }
                    SetMeasure::Reps { .. }
                    | SetMeasure::Duration { .. }
                    | SetMeasure::Cardio(_) => None,
                })
        })
        .map(|(grams, reps)| u64::from(grams) * u64::from(reps))
        .collect();
    (!set_volumes.is_empty()).then(|| set_volumes.iter().sum())
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::{FixedOffset, NaiveDate, TimeZone};

    use super::*;
    use crate::body_weight::BodyWeight;
    use crate::exercise::ExerciseId;
    use crate::measure::{Load, Weight};
    use crate::session::{EntryId, SessionId};
    use crate::set::WorkoutSet;
    use crate::text::NonEmptyText;

    fn builtin(id: &str) -> ExerciseId {
        ExerciseId::Builtin(id.to_owned())
    }

    fn reps(reps: u32, load_kg: u32) -> SetMeasure {
        SetMeasure::Reps {
            reps: NonZeroU32::new(reps).unwrap(),
            load: Load::from_grams(load_kg * 1_000).ok(),
        }
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    fn done_session(db: &mut Db, day: u32, sets: &[(ExerciseId, SetMeasure, bool)]) -> SessionId {
        let at = |hour| {
            FixedOffset::east_opt(3_600)
                .unwrap()
                .with_ymd_and_hms(2026, 10, day, hour, 0, 0)
                .unwrap()
        };
        let exercises: Vec<ExerciseId> = sets.iter().map(|(id, _, _)| id.clone()).collect();
        let id = db
            .start_new_session(NonEmptyText::new("Push").unwrap(), &exercises, at(18))
            .unwrap();
        for ((_, measure, warmup), entry) in sets.iter().zip(1..) {
            db.add_set(
                id,
                EntryId::new(entry),
                WorkoutSet::new(*measure, *warmup, None),
            )
            .unwrap();
        }
        db.finish_session(id, at(19)).unwrap();
        id
    }

    fn volume(db: &Db, id: SessionId) -> Option<u64> {
        session_volume_grams(db, db.session(id).unwrap())
    }

    #[test]
    fn counts_weighted_sets_without_warm_ups() {
        let mut db = Db::default();
        let bench = SetMeasure::WeightReps {
            weight: Weight::from_grams(80_000),
            reps: NonZeroU32::new(8).unwrap(),
        };
        let id = done_session(
            &mut db,
            20,
            &[
                (builtin("barbell_bench_press"), bench, false),
                (builtin("barbell_bench_press"), bench, true),
            ],
        );
        assert_eq!(volume(&db, id), Some(640_000));
    }

    #[test]
    fn bodyweight_sets_need_a_known_body_weight() {
        let mut db = Db::default();
        let id = done_session(&mut db, 20, &[(builtin("pull_up"), reps(10, 0), false)]);
        assert_eq!(volume(&db, id), None);
    }

    #[test]
    fn bodyweight_sets_add_the_load_to_the_body_weight_of_the_session_date() {
        let mut db = Db::default();
        db.record_body_weight(day(10), BodyWeight::from_grams(80_000).unwrap());
        db.record_body_weight(day(25), BodyWeight::from_grams(75_000).unwrap());
        let before = done_session(&mut db, 5, &[(builtin("pull_up"), reps(10, 0), false)]);
        let during = done_session(&mut db, 20, &[(builtin("pull_up"), reps(10, 10), false)]);
        let after = done_session(&mut db, 28, &[(builtin("pull_up"), reps(10, 0), false)]);
        assert_eq!(volume(&db, before), Some(800_000));
        assert_eq!(volume(&db, during), Some(900_000));
        assert_eq!(volume(&db, after), Some(750_000));
    }

    #[test]
    fn band_exercises_do_not_lift_the_body_weight() {
        let mut db = Db::default();
        db.record_body_weight(day(10), BodyWeight::from_grams(80_000).unwrap());
        let id = done_session(
            &mut db,
            20,
            &[(builtin("band_assisted_pull_up"), reps(10, 0), false)],
        );
        assert_eq!(volume(&db, id), None);
    }
}
