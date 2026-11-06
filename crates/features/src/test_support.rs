use std::num::NonZeroU32;

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use domain::{
    Category, Db, EntryId, Equipment, ExerciseId, ExerciseInfo, MeasureKind, MuscleGroup,
    NonEmptyText, SessionId, SetMeasure, Weight, WorkoutSet,
};

pub fn text(value: &str) -> NonEmptyText {
    NonEmptyText::new(value).unwrap()
}

pub fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
}

pub fn custom_info(name: &str, measure: MeasureKind) -> ExerciseInfo {
    ExerciseInfo::new(
        text(name),
        Category::Strength,
        MuscleGroup::Back,
        Equipment::Machine,
        measure,
    )
    .unwrap()
}

pub fn at(day: u32, hour: u32, minute: u32) -> DateTime<FixedOffset> {
    FixedOffset::east_opt(3_600)
        .unwrap()
        .with_ymd_and_hms(2026, 10, day, hour, minute, 0)
        .unwrap()
}

pub fn bench() -> ExerciseId {
    ExerciseId::Builtin("barbell_bench_press".to_owned())
}

pub fn pull_up() -> ExerciseId {
    ExerciseId::Builtin("pull_up".to_owned())
}

pub fn weight_reps(kg: u32, reps: u32) -> SetMeasure {
    SetMeasure::WeightReps {
        weight: Weight::from_grams(kg * 1_000),
        reps: NonZeroU32::new(reps).unwrap(),
    }
}

pub fn reps(reps: u32) -> SetMeasure {
    SetMeasure::Reps {
        reps: NonZeroU32::new(reps).unwrap(),
        load: None,
    }
}

// Started on October 30 at 18:00; entries numbered 1, 2… in exercise order.
pub fn start_session(db: &mut Db, exercises: &[ExerciseId]) -> SessionId {
    db.start_new_session(
        NonEmptyText::new("Push A").unwrap(),
        exercises,
        at(30, 18, 0),
    )
    .unwrap()
}

pub fn db_with_session_in_progress(exercises: &[ExerciseId]) -> (Db, SessionId) {
    let mut db = Db::default();
    let id = start_session(&mut db, exercises);
    (db, id)
}

pub fn add_set_at(db: &mut Db, session: SessionId, entry: u32, measure: SetMeasure, minute: u32) {
    let set = WorkoutSet::new(measure, false, Some(at(30, 18, minute)));
    db.add_set(session, EntryId::new(entry), set).unwrap();
}

// On the given October day, from 18:00 to 19:00; sets completed at 18:05, 18:10…
// Call before `start_session`: only one session can be in progress.
pub fn done_session(
    db: &mut Db,
    day: u32,
    exercises: &[(ExerciseId, Vec<SetMeasure>)],
) -> SessionId {
    let ids: Vec<ExerciseId> = exercises.iter().map(|(id, _)| id.clone()).collect();
    let session = db
        .start_new_session(NonEmptyText::new("Push A").unwrap(), &ids, at(day, 18, 0))
        .unwrap();
    let mut minute = 0;
    for ((_, sets), entry) in exercises.iter().zip(1..) {
        for measure in sets {
            minute += 5;
            let set = WorkoutSet::new(*measure, false, Some(at(day, 18, minute)));
            db.add_set(session, EntryId::new(entry), set).unwrap();
        }
    }
    db.finish_session(session, at(day, 19, 0)).unwrap();
    session
}
