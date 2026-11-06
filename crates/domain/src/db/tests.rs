use std::num::NonZeroU32;

use chrono::TimeZone;

use super::*;
use crate::body_weight::BodyWeightLog;
use crate::exercise::{Category, Equipment, MuscleGroup};
use crate::measure::{MeasureKind, Weight};
use crate::session::SessionExercise;

fn at(hour: u32, minute: u32) -> DateTime<FixedOffset> {
    FixedOffset::east_opt(3_600)
        .unwrap()
        .with_ymd_and_hms(2026, 10, 28, hour, minute, 0)
        .unwrap()
}

fn text(value: &str) -> NonEmptyText {
    NonEmptyText::new(value).unwrap()
}

fn bench() -> ExerciseId {
    ExerciseId::Builtin("barbell_bench_press".to_owned())
}

fn pull_up() -> ExerciseId {
    ExerciseId::Builtin("pull_up".to_owned())
}

fn weight_reps(kg: u32, reps: u32) -> SetMeasure {
    SetMeasure::WeightReps {
        weight: Weight::from_grams(kg * 1_000),
        reps: NonZeroU32::new(reps).unwrap(),
    }
}

fn reps(reps: u32) -> SetMeasure {
    SetMeasure::Reps {
        reps: NonZeroU32::new(reps).unwrap(),
        load: None,
    }
}

fn set(measure: SetMeasure, completed_at: Option<DateTime<FixedOffset>>) -> WorkoutSet {
    WorkoutSet::new(measure, false, completed_at)
}

fn machine_info(measure: MeasureKind) -> ExerciseInfo {
    ExerciseInfo::new(
        text("Rowing machine Technogym"),
        Category::Strength,
        MuscleGroup::Back,
        Equipment::Machine,
        measure,
    )
    .unwrap()
}

fn entry_ids(db: &Db, id: SessionId) -> Vec<u32> {
    db.session(id)
        .unwrap()
        .entries()
        .iter()
        .map(|entry| entry.entry_id().get())
        .collect()
}

fn exercise_ids(db: &Db, id: SessionId) -> Vec<ExerciseId> {
    db.session(id)
        .unwrap()
        .entries()
        .iter()
        .map(|entry| entry.exercise_id().clone())
        .collect()
}

fn db_with_bench_and_pull_up_in_progress() -> (Db, SessionId) {
    let mut db = Db::default();
    let id = db
        .start_new_session(text("Push A"), &[bench(), pull_up()], at(18, 0))
        .unwrap();
    (db, id)
}

fn db_with_one_bench_set_done() -> (Db, SessionId) {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    db.add_set(
        id,
        EntryId::new(1),
        set(weight_reps(80, 8), Some(at(18, 5))),
    )
    .unwrap();
    db.finish_session(id, at(19, 0)).unwrap();
    (db, id)
}

fn assert_refused_without_change<T>(
    db: &mut Db,
    expected: DbError,
    operation: impl FnOnce(&mut Db) -> Result<T, DbError>,
) {
    let before = db.clone();
    let result = operation(db);
    assert_eq!(result.map(|_| ()), Err(expected));
    assert_eq!(*db, before);
}

fn done_session(id: u32, entries: Vec<SessionExercise>) -> Session {
    Session::restore(
        SessionId::new(id),
        text("Push A"),
        SessionOrigin::Adhoc,
        None,
        SessionStatus::Done {
            started_at: at(18, 0),
            finished_at: at(19, 0),
        },
        None,
        entries,
    )
    .unwrap()
}

fn in_progress_session(id: u32) -> Session {
    Session::restore(
        SessionId::new(id),
        text("Jambes"),
        SessionOrigin::Adhoc,
        None,
        SessionStatus::InProgress {
            started_at: at(18, 0),
        },
        None,
        vec![],
    )
    .unwrap()
}

fn bench_entry(sets: Vec<WorkoutSet>) -> SessionExercise {
    SessionExercise::restore(EntryId::new(1), bench(), sets)
}

#[test]
fn restore_accepts_consistent_data() {
    let custom = CustomExercise::restore(1, machine_info(MeasureKind::WeightReps), true);
    let settings = BTreeMap::from([
        (
            ExerciseId::Custom(1),
            ExerciseSettings::new(text("Siège cran 4")),
        ),
        (bench(), ExerciseSettings::new(text("Banc plat"))),
    ]);
    let sessions = vec![
        done_session(1, vec![bench_entry(vec![set(weight_reps(80, 8), None)])]),
        in_progress_session(2),
    ];
    let preferences = Preferences::new(
        Some(WeeklyGoal::new(3).unwrap()),
        Some(text("Thomas")),
        BodyWeightLog::default(),
    );

    let db = Db::restore(vec![custom], settings, sessions, preferences).unwrap();

    assert_eq!(db.custom_exercises().len(), 1);
    assert_eq!(db.sessions().len(), 2);
    assert_eq!(
        db.session_in_progress().map(Session::id),
        Some(SessionId::new(2))
    );
    assert_eq!(db.preferences().weekly_goal().map(WeeklyGoal::get), Some(3));
    assert_eq!(db.preferences().first_name(), Some(&text("Thomas")));
}

#[test]
fn restore_rejects_duplicate_custom_exercise_ids() {
    let exercises = vec![
        CustomExercise::restore(1, machine_info(MeasureKind::WeightReps), false),
        CustomExercise::restore(1, machine_info(MeasureKind::Reps), false),
    ];
    assert_eq!(
        Db::restore(exercises, BTreeMap::new(), vec![], Preferences::default()),
        Err(IntegrityError::DuplicateCustomExerciseId(1))
    );
}

#[test]
fn restore_rejects_duplicate_session_ids() {
    let sessions = vec![done_session(1, vec![]), done_session(1, vec![])];
    assert_eq!(
        Db::restore(vec![], BTreeMap::new(), sessions, Preferences::default()),
        Err(IntegrityError::DuplicateSessionId(SessionId::new(1)))
    );
}

#[test]
fn restore_rejects_several_sessions_in_progress() {
    let sessions = vec![in_progress_session(1), in_progress_session(2)];
    assert_eq!(
        Db::restore(vec![], BTreeMap::new(), sessions, Preferences::default()),
        Err(IntegrityError::SeveralSessionsInProgress)
    );
}

#[test]
fn restore_rejects_unknown_exercise_in_session() {
    let entry = SessionExercise::restore(EntryId::new(1), ExerciseId::Custom(9), vec![]);
    let sessions = vec![done_session(1, vec![entry])];
    assert_eq!(
        Db::restore(vec![], BTreeMap::new(), sessions, Preferences::default()),
        Err(IntegrityError::UnknownExercise(ExerciseId::Custom(9)))
    );
}

#[test]
fn restore_rejects_settings_of_unknown_exercise() {
    let unknown = ExerciseId::Builtin("unknown".to_owned());
    let settings = BTreeMap::from([(unknown.clone(), ExerciseSettings::new(text("Cran 2")))]);
    assert_eq!(
        Db::restore(vec![], settings, vec![], Preferences::default()),
        Err(IntegrityError::UnknownExercise(unknown))
    );
}

#[test]
fn restore_rejects_set_with_wrong_measure() {
    let sessions = vec![done_session(
        1,
        vec![bench_entry(vec![set(reps(10), None)])],
    )];
    assert_eq!(
        Db::restore(vec![], BTreeMap::new(), sessions, Preferences::default()),
        Err(IntegrityError::MeasureMismatch {
            session: SessionId::new(1),
            entry: EntryId::new(1),
        })
    );
}

#[test]
fn exercise_resolves_builtin_and_custom_ids() {
    let mut db = Db::default();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    assert_eq!(
        db.exercise(&bench()).map(|info| info.name().as_str()),
        Some("Développé couché barre")
    );
    assert_eq!(
        db.exercise(&custom).map(|info| info.name().as_str()),
        Some("Rowing machine Technogym")
    );
    assert!(db.exercise(&ExerciseId::Custom(42)).is_none());
    assert!(
        db.exercise(&ExerciseId::Builtin("unknown".to_owned()))
            .is_none()
    );
}

#[test]
fn custom_exercises_get_increasing_ids() {
    let mut db = Db::default();
    let first = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    let second = db
        .add_custom_exercise(machine_info(MeasureKind::Reps))
        .unwrap();
    assert_eq!(first, ExerciseId::Custom(1));
    assert_eq!(second, ExerciseId::Custom(2));
}

#[test]
fn unused_custom_exercise_can_change_measure() {
    let mut db = Db::default();
    db.add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    db.update_custom_exercise(1, machine_info(MeasureKind::Reps))
        .unwrap();
    assert_eq!(
        db.custom_exercise(1).map(|e| e.info().measure()),
        Some(MeasureKind::Reps)
    );
}

#[test]
fn used_custom_exercise_keeps_its_measure() {
    let mut db = Db::default();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    db.plan_session(text("Dos"), None, &[custom]).unwrap();

    assert_refused_without_change(&mut db, DbError::MeasureLockedByUse, |db| {
        db.update_custom_exercise(1, machine_info(MeasureKind::Reps))
    });

    let renamed = ExerciseInfo::new(
        text("Rowing machine"),
        Category::Strength,
        MuscleGroup::Back,
        Equipment::Machine,
        MeasureKind::WeightReps,
    )
    .unwrap();
    db.update_custom_exercise(1, renamed).unwrap();
    assert_eq!(
        db.custom_exercise(1).map(|e| e.info().name().as_str()),
        Some("Rowing machine")
    );
}

#[test]
fn updating_unknown_custom_exercise_is_refused() {
    let mut db = Db::default();
    assert_refused_without_change(
        &mut db,
        DbError::ExerciseNotFound(ExerciseId::Custom(1)),
        |db| db.update_custom_exercise(1, machine_info(MeasureKind::Reps)),
    );
}

#[test]
fn unused_custom_exercise_is_deleted_with_its_settings() {
    let mut db = Db::default();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    db.set_exercise_settings(custom.clone(), Some(ExerciseSettings::new(text("Cran 4"))))
        .unwrap();

    assert_eq!(db.remove_custom_exercise(1), Ok(Removed::Deleted));
    assert!(db.custom_exercises().is_empty());
    assert!(db.exercise_settings(&custom).is_none());
}

#[test]
fn used_custom_exercise_is_archived_then_unarchived() {
    let mut db = Db::default();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    db.plan_session(text("Dos"), None, std::slice::from_ref(&custom))
        .unwrap();

    assert_eq!(db.remove_custom_exercise(1), Ok(Removed::Archived));
    assert!(db.custom_exercise(1).unwrap().is_archived());
    assert!(db.exercise(&custom).is_some());

    db.unarchive_custom_exercise(1).unwrap();
    assert!(!db.custom_exercise(1).unwrap().is_archived());
}

#[test]
fn removing_unknown_custom_exercise_is_refused() {
    let mut db = Db::default();
    assert_refused_without_change(
        &mut db,
        DbError::ExerciseNotFound(ExerciseId::Custom(1)),
        |db| db.remove_custom_exercise(1),
    );
    assert_refused_without_change(
        &mut db,
        DbError::ExerciseNotFound(ExerciseId::Custom(1)),
        |db| db.unarchive_custom_exercise(1),
    );
}

#[test]
fn settings_are_set_and_cleared() {
    let mut db = Db::default();
    db.set_exercise_settings(bench(), Some(ExerciseSettings::new(text("Banc cran 2"))))
        .unwrap();
    assert_eq!(
        db.exercise_settings(&bench()).map(|s| s.text().as_str()),
        Some("Banc cran 2")
    );
    db.set_exercise_settings(bench(), None).unwrap();
    assert!(db.exercise_settings(&bench()).is_none());
    assert!(db.all_exercise_settings().is_empty());
}

#[test]
fn settings_of_unknown_exercise_are_refused() {
    let mut db = Db::default();
    assert_refused_without_change(
        &mut db,
        DbError::ExerciseNotFound(ExerciseId::Custom(1)),
        |db| {
            db.set_exercise_settings(
                ExerciseId::Custom(1),
                Some(ExerciseSettings::new(text("Cran 4"))),
            )
        },
    );
}

#[test]
fn weekly_goal_is_configurable() {
    let mut db = Db::default();
    db.set_weekly_goal(Some(WeeklyGoal::new(4).unwrap()));
    assert_eq!(db.preferences().weekly_goal().map(WeeklyGoal::get), Some(4));
    db.set_weekly_goal(None);
    assert_eq!(db.preferences().weekly_goal(), None);
}

#[test]
fn first_name_can_be_set_and_cleared() {
    let mut db = Db::default();
    db.set_first_name(Some(text("Thomas")));
    assert_eq!(db.preferences().first_name(), Some(&text("Thomas")));
    db.set_first_name(None);
    assert_eq!(db.preferences().first_name(), None);
}

#[test]
fn plan_session_creates_planned_session_with_entries() {
    let mut db = Db::default();
    let date = NaiveDate::from_ymd_opt(2026, 11, 3).unwrap();
    let id = db
        .plan_session(text("Push A"), Some(date), &[bench(), pull_up(), bench()])
        .unwrap();

    let session = db.session(id).unwrap();
    assert_eq!(id, SessionId::new(1));
    assert_eq!(session.name().as_str(), "Push A");
    assert_eq!(session.origin(), SessionOrigin::Planned);
    assert_eq!(session.status(), &SessionStatus::Planned);
    assert_eq!(session.planned_for(), Some(date));
    assert_eq!(entry_ids(&db, id), [1, 2, 3]);
    assert_eq!(exercise_ids(&db, id), [bench(), pull_up(), bench()]);
}

#[test]
fn plan_session_refuses_unknown_or_archived_exercise() {
    let mut db = Db::default();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    db.plan_session(text("Dos"), None, std::slice::from_ref(&custom))
        .unwrap();
    db.remove_custom_exercise(1).unwrap();

    assert_refused_without_change(&mut db, DbError::ExerciseArchived(custom.clone()), |db| {
        db.plan_session(text("Dos"), None, &[bench(), custom])
    });
    let unknown = ExerciseId::Builtin("unknown".to_owned());
    assert_refused_without_change(&mut db, DbError::ExerciseNotFound(unknown.clone()), |db| {
        db.plan_session(text("Dos"), None, &[unknown])
    });
}

#[test]
fn start_new_session_creates_adhoc_session_in_progress() {
    let (db, id) = db_with_bench_and_pull_up_in_progress();
    let session = db.session(id).unwrap();
    assert_eq!(session.origin(), SessionOrigin::Adhoc);
    assert_eq!(
        session.status(),
        &SessionStatus::InProgress {
            started_at: at(18, 0)
        }
    );
    assert_eq!(db.session_in_progress().map(Session::id), Some(id));
}

#[test]
fn only_one_session_can_be_in_progress() {
    let (mut db, _) = db_with_bench_and_pull_up_in_progress();
    let planned = db.plan_session(text("Jambes"), None, &[]).unwrap();

    assert_refused_without_change(&mut db, DbError::AnotherSessionInProgress, |db| {
        db.start_new_session(text("Jambes"), &[], at(18, 30))
    });
    assert_refused_without_change(&mut db, DbError::AnotherSessionInProgress, |db| {
        db.start_planned_session(planned, at(18, 30))
    });
}

#[test]
fn start_planned_session_starts_it() {
    let mut db = Db::default();
    let id = db.plan_session(text("Push A"), None, &[bench()]).unwrap();
    db.start_planned_session(id, at(18, 0)).unwrap();
    assert_eq!(
        db.session(id).unwrap().status(),
        &SessionStatus::InProgress {
            started_at: at(18, 0)
        }
    );
}

#[test]
fn only_planned_sessions_can_be_started() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    assert_refused_without_change(&mut db, DbError::NotPlanned, |db| {
        db.start_planned_session(id, at(18, 30))
    });
    assert_refused_without_change(&mut db, DbError::SessionNotFound(SessionId::new(9)), |db| {
        db.start_planned_session(SessionId::new(9), at(18, 30))
    });
}

#[test]
fn finish_session_removes_entries_without_sets() {
    let (db, id) = db_with_one_bench_set_done();
    let session = db.session(id).unwrap();
    assert_eq!(
        session.status(),
        &SessionStatus::Done {
            started_at: at(18, 0),
            finished_at: at(19, 0),
        }
    );
    assert_eq!(entry_ids(&db, id), [1]);
    assert!(db.session_in_progress().is_none());
}

#[test]
fn finish_session_requires_at_least_one_set() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    assert_refused_without_change(&mut db, DbError::FinishWithoutSets, |db| {
        db.finish_session(id, at(19, 0))
    });
}

#[test]
fn finish_session_cannot_end_before_start() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    db.add_set(id, EntryId::new(1), set(weight_reps(80, 8), None))
        .unwrap();
    assert_refused_without_change(&mut db, DbError::FinishedBeforeStarted, |db| {
        db.finish_session(id, at(17, 0))
    });
}

#[test]
fn only_sessions_in_progress_can_be_finished() {
    let (mut db, id) = db_with_one_bench_set_done();
    assert_refused_without_change(&mut db, DbError::NotInProgress, |db| {
        db.finish_session(id, at(20, 0))
    });
}

#[test]
fn cancel_planned_session_returns_to_planned() {
    let mut db = Db::default();
    let id = db.plan_session(text("Push A"), None, &[bench()]).unwrap();
    db.start_planned_session(id, at(18, 0)).unwrap();

    assert_eq!(db.cancel_session(id), Ok(Cancelled::BackToPlanned));
    assert_eq!(db.session(id).unwrap().status(), &SessionStatus::Planned);
    assert_eq!(exercise_ids(&db, id), [bench()]);
}

#[test]
fn cancel_adhoc_session_deletes_it() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    assert_eq!(db.cancel_session(id), Ok(Cancelled::Deleted));
    assert!(db.session(id).is_none());
}

#[test]
fn cancel_is_refused_once_a_set_is_recorded() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    db.add_set(id, EntryId::new(1), set(weight_reps(80, 8), None))
        .unwrap();
    assert_refused_without_change(&mut db, DbError::CancelWithSets, |db| db.cancel_session(id));
}

#[test]
fn only_sessions_in_progress_can_be_cancelled() {
    let (mut db, id) = db_with_one_bench_set_done();
    assert_refused_without_change(&mut db, DbError::NotInProgress, |db| db.cancel_session(id));
}

#[test]
fn delete_session_removes_it() {
    let (mut db, id) = db_with_one_bench_set_done();
    db.delete_session(id).unwrap();
    assert!(db.sessions().is_empty());
    assert_refused_without_change(&mut db, DbError::SessionNotFound(id), |db| {
        db.delete_session(id)
    });
}

#[test]
fn session_ids_are_max_plus_one() {
    let mut db = Db::default();
    let first = db.plan_session(text("A"), None, &[]).unwrap();
    let second = db.plan_session(text("B"), None, &[]).unwrap();
    db.delete_session(first).unwrap();
    let third = db.plan_session(text("C"), None, &[]).unwrap();
    assert_eq!([first, second, third].map(SessionId::get), [1, 2, 3]);
}

#[test]
fn session_template_copies_name_and_exercises_of_a_done_session() {
    let mut db = Db::default();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    let id = db
        .start_new_session(
            text("Push A"),
            &[bench(), custom, pull_up(), bench()],
            at(18, 0),
        )
        .unwrap();
    for entry in 1..=4 {
        let measure = if entry == 3 {
            reps(10)
        } else {
            weight_reps(60, 10)
        };
        db.add_set(id, EntryId::new(entry), set(measure, None))
            .unwrap();
    }
    db.finish_session(id, at(19, 0)).unwrap();
    db.remove_custom_exercise(1).unwrap();

    let (name, exercises) = db.session_template(id).unwrap();
    assert_eq!(name.as_str(), "Push A");
    assert_eq!(exercises, [bench(), pull_up(), bench()]);
}

#[test]
fn session_template_requires_a_done_session() {
    let (db, id) = db_with_bench_and_pull_up_in_progress();
    assert_eq!(db.session_template(id), Err(DbError::NotDone));
}

#[test]
fn session_details_can_be_edited() {
    let (mut db, id) = db_with_one_bench_set_done();
    let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    db.rename_session(id, text("Push B")).unwrap();
    db.set_session_planned_for(id, Some(date)).unwrap();
    db.set_session_notes(id, Some(text("Bonne forme"))).unwrap();

    let session = db.session(id).unwrap();
    assert_eq!(session.name().as_str(), "Push B");
    assert_eq!(session.planned_for(), Some(date));
    assert_eq!(
        session.notes().map(NonEmptyText::as_str),
        Some("Bonne forme")
    );
    assert_refused_without_change(&mut db, DbError::SessionNotFound(SessionId::new(9)), |db| {
        db.rename_session(SessionId::new(9), text("X"))
    });
}

#[test]
fn done_session_times_can_be_corrected() {
    let (mut db, id) = db_with_one_bench_set_done();
    db.update_session_times(id, at(17, 30), at(18, 45)).unwrap();
    assert_eq!(
        db.session(id).unwrap().status(),
        &SessionStatus::Done {
            started_at: at(17, 30),
            finished_at: at(18, 45),
        }
    );
    assert_refused_without_change(&mut db, DbError::FinishedBeforeStarted, |db| {
        db.update_session_times(id, at(19, 0), at(18, 0))
    });
}

#[test]
fn only_done_session_times_can_be_corrected() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    assert_refused_without_change(&mut db, DbError::NotDone, |db| {
        db.update_session_times(id, at(17, 0), at(18, 0))
    });
}

#[test]
fn entries_can_be_added_removed_and_moved() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    let added = db.add_entry(id, bench()).unwrap();
    assert_eq!(added, EntryId::new(3));
    assert_eq!(exercise_ids(&db, id), [bench(), pull_up(), bench()]);

    db.move_entry(id, added, 0).unwrap();
    assert_eq!(entry_ids(&db, id), [3, 1, 2]);

    db.remove_entry(id, EntryId::new(1)).unwrap();
    assert_eq!(entry_ids(&db, id), [3, 2]);

    assert_refused_without_change(&mut db, DbError::EntryNotFound(EntryId::new(1)), |db| {
        db.remove_entry(id, EntryId::new(1))
    });
    assert_refused_without_change(&mut db, DbError::InvalidPosition(2), |db| {
        db.move_entry(id, added, 2)
    });
}

#[test]
fn retired_pool_exercise_cannot_be_added_but_past_sessions_still_load() {
    let roller = ExerciseId::Builtin("foam_roll_quads".to_owned());
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    assert_refused_without_change(&mut db, DbError::ExerciseArchived(roller.clone()), |db| {
        db.add_entry(id, roller.clone())
    });

    let past = done_session(
        1,
        vec![SessionExercise::restore(
            EntryId::new(1),
            roller.clone(),
            vec![],
        )],
    );
    let restored =
        Db::restore(vec![], BTreeMap::new(), vec![past], Preferences::default()).unwrap();
    assert!(restored.exercise(&roller).is_some());
}

#[test]
fn archived_exercise_cannot_be_added() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    let custom = db
        .add_custom_exercise(machine_info(MeasureKind::WeightReps))
        .unwrap();
    db.add_entry(id, custom.clone()).unwrap();
    db.remove_custom_exercise(1).unwrap();

    assert_refused_without_change(&mut db, DbError::ExerciseArchived(custom.clone()), |db| {
        db.add_entry(id, custom)
    });
}

#[test]
fn sets_are_added_with_their_completion_time() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    db.add_set(
        id,
        EntryId::new(1),
        set(weight_reps(80, 8), Some(at(18, 5))),
    )
    .unwrap();
    db.add_set(id, EntryId::new(2), set(reps(10), Some(at(18, 8))))
        .unwrap();

    let session = db.session(id).unwrap();
    let sets = session.entry(EntryId::new(1)).unwrap().sets();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].measure(), &weight_reps(80, 8));
    assert_eq!(sets[0].completed_at(), Some(at(18, 5)));
}

#[test]
fn set_measure_must_match_the_exercise() {
    let (mut db, id) = db_with_bench_and_pull_up_in_progress();
    let mismatch = DbError::MeasureMismatch {
        expected: MeasureKind::WeightReps,
        found: MeasureKind::Reps,
    };
    assert_refused_without_change(&mut db, mismatch, |db| {
        db.add_set(id, EntryId::new(1), set(reps(10), None))
    });
}

#[test]
fn planned_session_cannot_contain_sets() {
    let mut db = Db::default();
    let id = db.plan_session(text("Push A"), None, &[bench()]).unwrap();
    assert_refused_without_change(&mut db, DbError::SetsNotAllowedInPlannedSession, |db| {
        db.add_set(id, EntryId::new(1), set(weight_reps(80, 8), None))
    });
}

#[test]
fn done_session_accepts_corrections() {
    let (mut db, id) = db_with_one_bench_set_done();
    db.add_set(id, EntryId::new(1), set(weight_reps(60, 12), None))
        .unwrap();
    let sets = db
        .session(id)
        .unwrap()
        .entry(EntryId::new(1))
        .unwrap()
        .sets();
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[1].completed_at(), None);
}

#[test]
fn update_set_keeps_completion_time_and_checks_measure() {
    let (mut db, id) = db_with_one_bench_set_done();
    db.update_set(id, EntryId::new(1), 0, weight_reps(82, 6), true)
        .unwrap();
    let sets = db
        .session(id)
        .unwrap()
        .entry(EntryId::new(1))
        .unwrap()
        .sets();
    assert_eq!(sets[0].measure(), &weight_reps(82, 6));
    assert!(sets[0].is_warmup());
    assert_eq!(sets[0].completed_at(), Some(at(18, 5)));

    let mismatch = DbError::MeasureMismatch {
        expected: MeasureKind::WeightReps,
        found: MeasureKind::Reps,
    };
    assert_refused_without_change(&mut db, mismatch, |db| {
        db.update_set(id, EntryId::new(1), 0, reps(10), false)
    });
    assert_refused_without_change(
        &mut db,
        DbError::SetNotFound {
            entry: EntryId::new(1),
            index: 1,
        },
        |db| db.update_set(id, EntryId::new(1), 1, weight_reps(80, 8), false),
    );
}

#[test]
fn remove_set_removes_it() {
    let (mut db, id) = db_with_one_bench_set_done();
    db.remove_set(id, EntryId::new(1), 0).unwrap();
    assert!(
        db.session(id)
            .unwrap()
            .entry(EntryId::new(1))
            .unwrap()
            .sets()
            .is_empty()
    );
    assert_refused_without_change(
        &mut db,
        DbError::SetNotFound {
            entry: EntryId::new(1),
            index: 0,
        },
        |db| db.remove_set(id, EntryId::new(1), 0),
    );
}

#[test]
fn exercise_is_used_once_referenced_by_any_session() {
    let mut db = Db::default();
    assert!(!db.is_exercise_used(&bench()));
    db.plan_session(text("Push A"), None, &[bench()]).unwrap();
    assert!(db.is_exercise_used(&bench()));
    assert!(!db.is_exercise_used(&pull_up()));
}
