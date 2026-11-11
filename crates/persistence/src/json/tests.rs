use std::num::NonZeroU32;

use chrono::{DateTime, NaiveDate};
use domain::{
    CardioMeasure, EntryId, ExerciseId, IntegrityError, Load, MeasureKind, Meters, Seconds,
    SessionId, SessionOrigin, SessionStatus, SetMeasure, WeeklyGoal, Weight,
};
use serde_json::{Value, json};

use super::*;
use crate::contract::check_codec_contract;
use crate::mapping::MappingError;

const FIXTURE_V1: &str = include_str!("../../fixtures/v1.json");

fn fixture_db() -> Db {
    JsonCodec.decode(FIXTURE_V1.as_bytes()).unwrap()
}

fn decode_modified_fixture(modify: impl FnOnce(&mut Value)) -> Result<Db, DecodeError> {
    let mut fixture: Value = serde_json::from_str(FIXTURE_V1).unwrap();
    modify(&mut fixture);
    JsonCodec.decode(&serde_json::to_vec(&fixture).unwrap())
}

fn invalid_content(error: MappingError) -> Result<Db, DecodeError> {
    Err(DecodeError::InvalidContent(error))
}

fn time(rfc3339: &str) -> DateTime<chrono::FixedOffset> {
    DateTime::parse_from_rfc3339(rfc3339).unwrap()
}

#[test]
fn satisfies_codec_contract() {
    check_codec_contract(&JsonCodec, &fixture_db());
}

#[test]
fn encoding_keeps_the_frozen_v1_format() {
    let encoded: Value = serde_json::from_slice(&JsonCodec.encode(&fixture_db()).unwrap()).unwrap();
    let fixture: Value = serde_json::from_str(FIXTURE_V1).unwrap();
    assert_eq!(encoded, fixture);
}

#[test]
fn fixture_v1_is_fully_decoded() {
    let db = fixture_db();

    assert_eq!(db.preferences().weekly_goal().map(WeeklyGoal::get), Some(4));

    let rowing = db.custom_exercise(1).unwrap();
    assert_eq!(rowing.info().name().as_str(), "Rowing machine Technogym");
    assert_eq!(rowing.info().measure(), MeasureKind::WeightReps);
    assert!(!rowing.is_archived());
    assert!(db.custom_exercise(2).unwrap().is_archived());
    assert_eq!(
        db.exercise_settings(&ExerciseId::Builtin("leg_press".to_owned()))
            .map(|settings| settings.text().as_str()),
        Some("Dossier cran 2, pieds hauts")
    );

    let push = db.session(SessionId::new(42)).unwrap();
    assert_eq!(push.name().as_str(), "Push A");
    assert_eq!(push.origin(), SessionOrigin::Planned);
    assert_eq!(push.planned_for(), NaiveDate::from_ymd_opt(2026, 10, 28));
    assert_eq!(
        push.status(),
        &SessionStatus::Done {
            started_at: time("2026-10-28T18:02:00+01:00"),
            finished_at: time("2026-10-28T19:10:00+01:00"),
        }
    );
    assert_eq!(
        push.notes().map(|notes| notes.as_str()),
        Some("Bonne forme")
    );
    assert_eq!(push.entries().len(), 6);

    let bench_sets = push.entry(EntryId::new(1)).unwrap().sets();
    assert_eq!(
        bench_sets[0].measure(),
        &SetMeasure::WeightReps {
            weight: Weight::from_grams(60_000),
            reps: NonZeroU32::new(10).unwrap(),
        }
    );
    assert!(bench_sets[0].is_warmup());
    assert_eq!(
        bench_sets[0].completed_at(),
        Some(time("2026-10-28T18:06:00+01:00"))
    );
    assert_eq!(
        push.entry(EntryId::new(2)).unwrap().sets()[0].completed_at(),
        None
    );
    assert_eq!(
        push.entry(EntryId::new(3)).unwrap().sets()[1].measure(),
        &SetMeasure::Reps {
            reps: NonZeroU32::new(6).unwrap(),
            load: Some(Load::from_grams(10_000).unwrap()),
        }
    );
    assert_eq!(
        push.entry(EntryId::new(6)).unwrap().sets()[0].measure(),
        &SetMeasure::Cardio(CardioMeasure::new(
            Seconds::new(1_200).unwrap(),
            Some(Meters::new(5_000).unwrap()),
            None,
            Some(6),
            None,
        ))
    );

    assert_eq!(
        db.session(SessionId::new(43)).unwrap().status(),
        &SessionStatus::Planned
    );
    assert_eq!(
        db.session_in_progress().map(|session| session.id()),
        Some(SessionId::new(44))
    );
}

#[test]
fn body_weights_are_an_optional_compatible_addition() {
    assert_eq!(fixture_db().preferences().body_weights().latest(), None);

    let weighed = decode_modified_fixture(|db| {
        db["preferences"]["body_weights"] = json!([
            {"date": "2026-10-01", "weight_g": 80000},
            {"date": "2026-10-20", "weight_g": 78500}
        ]);
    })
    .unwrap();
    let date = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
    let in_effect = weighed.preferences().body_weights().on(date);
    assert_eq!(in_effect.map(domain::BodyWeight::grams), Some(80_000));
    let encoded: Value = serde_json::from_slice(&JsonCodec.encode(&weighed).unwrap()).unwrap();
    assert_eq!(
        encoded["preferences"]["body_weights"][1],
        json!({"date": "2026-10-20", "weight_g": 78500})
    );

    assert_eq!(
        decode_modified_fixture(|db| {
            db["preferences"]["body_weights"] = json!([{"date": "2026-10-01", "weight_g": 5}]);
        }),
        invalid_content(MappingError::BodyWeightOutOfRange(5))
    );
}

#[test]
fn first_name_is_an_optional_compatible_addition() {
    assert_eq!(fixture_db().preferences().first_name(), None);

    let named =
        decode_modified_fixture(|db| db["preferences"]["first_name"] = json!("Thomas")).unwrap();
    assert_eq!(
        named.preferences().first_name().map(|name| name.as_str()),
        Some("Thomas")
    );
    let encoded: Value = serde_json::from_slice(&JsonCodec.encode(&named).unwrap()).unwrap();
    assert_eq!(encoded["preferences"]["first_name"], json!("Thomas"));

    let blank =
        decode_modified_fixture(|db| db["preferences"]["first_name"] = json!("  ")).unwrap();
    assert_eq!(blank.preferences().first_name(), None);
}

#[test]
fn rejects_unsupported_schema_version() {
    assert_eq!(
        decode_modified_fixture(|db| db["schema_version"] = json!(2)),
        Err(DecodeError::UnsupportedSchemaVersion(2))
    );
}

#[test]
fn rejects_malformed_json() {
    assert!(matches!(
        decode_modified_fixture(
            |db| db["sessions"][0]["exercises"][0]["sets"][0]["kind"] = json!("weight")
        ),
        Err(DecodeError::Malformed { .. })
    ));
    assert!(matches!(
        decode_modified_fixture(|db| db["custom_exercises"][0]["muscle_group"] = json!("wings")),
        Err(DecodeError::Malformed { .. })
    ));
    assert!(matches!(
        JsonCodec.decode(br#"{"preferences": {}}"#),
        Err(DecodeError::Malformed { .. })
    ));
}

#[test]
fn rejects_out_of_range_values() {
    assert_eq!(
        decode_modified_fixture(
            |db| db["sessions"][0]["exercises"][0]["sets"][0]["reps"] = json!(0)
        ),
        invalid_content(MappingError::ZeroValue)
    );
    assert_eq!(
        decode_modified_fixture(|db| db["preferences"]["weekly_goal"] = json!(15)),
        invalid_content(MappingError::WeeklyGoalOutOfRange(15))
    );
    assert_eq!(
        decode_modified_fixture(|db| db["sessions"][0]["name"] = json!("  ")),
        invalid_content(MappingError::EmptyText)
    );
}

#[test]
fn rejects_invalid_exercise_ids() {
    assert_eq!(
        decode_modified_fixture(|db| db["exercise_settings"]["custom-01"] = json!("Cran 1")),
        invalid_content(MappingError::InvalidExerciseId("custom-01".to_owned()))
    );
    assert_eq!(
        decode_modified_fixture(|db| db["custom_exercises"][0]["id"] = json!("rowing")),
        invalid_content(MappingError::InvalidExerciseId("rowing".to_owned()))
    );
}

#[test]
fn rejects_status_inconsistent_with_times() {
    assert_eq!(
        decode_modified_fixture(|db| db["sessions"][0]["finished_at"] = Value::Null),
        invalid_content(MappingError::InconsistentSessionStatus(42))
    );
    assert_eq!(
        decode_modified_fixture(
            |db| db["sessions"][1]["started_at"] = json!("2026-11-03T18:00:00+01:00")
        ),
        invalid_content(MappingError::InconsistentSessionStatus(43))
    );
}

#[test]
fn rejects_forbidden_category_and_measure() {
    assert!(matches!(
        decode_modified_fixture(|db| db["custom_exercises"][0]["category"] = json!("stretching")),
        Err(DecodeError::InvalidContent(
            MappingError::MeasureNotAllowed(_)
        ))
    ));
}

#[test]
fn rejects_data_breaking_domain_invariants() {
    assert_eq!(
        decode_modified_fixture(
            |db| db["sessions"][0]["exercises"][0]["exercise_id"] = json!("unknown")
        ),
        invalid_content(MappingError::Integrity(IntegrityError::UnknownExercise(
            ExerciseId::Builtin("unknown".to_owned())
        )))
    );
    assert_eq!(
        decode_modified_fixture(|db| {
            db["sessions"][0]["exercises"][0]["sets"][0] = json!({ "kind": "reps", "reps": 10, "load_g": null, "warmup": false, "completed_at": null });
        }),
        invalid_content(MappingError::Integrity(IntegrityError::MeasureMismatch {
            session: SessionId::new(42),
            entry: EntryId::new(1),
        }))
    );
    assert_eq!(
        decode_modified_fixture(|db| db["sessions"][0]["status"] = json!("in_progress")),
        invalid_content(MappingError::InconsistentSessionStatus(42))
    );
}
