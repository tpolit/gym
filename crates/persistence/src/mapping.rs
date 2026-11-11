use std::collections::BTreeMap;
use std::num::NonZeroU32;

use chrono::{DateTime, FixedOffset};
use domain::{
    BodyWeight, BodyWeightLog, CardioMeasure, Category, CustomExercise, Db, EntryId, Equipment,
    ExerciseId, ExerciseInfo, ExerciseInfoError, ExerciseSettings, IntegrityError, Kcal, Load,
    MeasureKind, Meters, MuscleGroup, NonEmptyText, Permille, Preferences, Seconds, Session,
    SessionExercise, SessionId, SessionOrigin, SessionStatus, SetMeasure, TextError, ValueError,
    WeeklyGoal, Weight, WorkoutSet,
};

use crate::dto::v1::{
    BodyWeightV1, CategoryV1, CustomExerciseV1, DbV1, EquipmentV1, MeasureKindV1, MuscleGroupV1,
    PreferencesV1, SCHEMA_VERSION, SessionExerciseV1, SessionOriginV1, SessionStatusV1, SessionV1,
    WorkoutSetV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingError {
    InvalidExerciseId(String),
    EmptyText,
    ZeroValue,
    WeeklyGoalOutOfRange(u8),
    BodyWeightOutOfRange(u32),
    MeasureNotAllowed(ExerciseInfoError),
    InconsistentSessionStatus(u32),
    Integrity(IntegrityError),
}

impl From<TextError> for MappingError {
    fn from(_: TextError) -> Self {
        Self::EmptyText
    }
}

impl From<ValueError> for MappingError {
    fn from(_: ValueError) -> Self {
        Self::ZeroValue
    }
}

impl From<ExerciseInfoError> for MappingError {
    fn from(error: ExerciseInfoError) -> Self {
        Self::MeasureNotAllowed(error)
    }
}

impl From<IntegrityError> for MappingError {
    fn from(error: IntegrityError) -> Self {
        Self::Integrity(error)
    }
}

const CUSTOM_EXERCISE_ID_PREFIX: &str = "custom-";

pub fn db_to_dto(db: &Db) -> DbV1 {
    DbV1 {
        schema_version: SCHEMA_VERSION,
        preferences: PreferencesV1 {
            weekly_goal: db.preferences().weekly_goal().map(WeeklyGoal::get),
            first_name: db.preferences().first_name().map(NonEmptyText::to_string),
            body_weights: db
                .preferences()
                .body_weights()
                .entries()
                .map(|(date, weight)| BodyWeightV1 {
                    date,
                    weight_g: weight.grams(),
                })
                .collect(),
        },
        custom_exercises: db
            .custom_exercises()
            .iter()
            .map(custom_exercise_to_dto)
            .collect(),
        exercise_settings: db
            .all_exercise_settings()
            .iter()
            .map(|(id, settings)| (exercise_id_to_dto(id), settings.text().to_string()))
            .collect(),
        sessions: db.sessions().iter().map(session_to_dto).collect(),
    }
}

pub fn db_from_dto(dto: DbV1) -> Result<Db, MappingError> {
    let weekly_goal = dto
        .preferences
        .weekly_goal
        .map(|goal| WeeklyGoal::new(goal).map_err(|_| MappingError::WeeklyGoalOutOfRange(goal)))
        .transpose()?;
    let first_name = dto.preferences.first_name.and_then(NonEmptyText::optional);
    let body_weights = dto
        .preferences
        .body_weights
        .into_iter()
        .map(|entry| {
            BodyWeight::from_grams(entry.weight_g)
                .map(|weight| (entry.date, weight))
                .map_err(|_| MappingError::BodyWeightOutOfRange(entry.weight_g))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let custom_exercises = dto
        .custom_exercises
        .into_iter()
        .map(custom_exercise_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    let exercise_settings = dto
        .exercise_settings
        .into_iter()
        .map(|(id, text)| {
            Ok((
                exercise_id_from_dto(id)?,
                ExerciseSettings::new(NonEmptyText::new(text)?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, MappingError>>()?;
    let sessions = dto
        .sessions
        .into_iter()
        .map(session_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Db::restore(
        custom_exercises,
        exercise_settings,
        sessions,
        Preferences::new(
            weekly_goal,
            first_name,
            BodyWeightLog::from_entries(body_weights),
        ),
    )?)
}

fn exercise_id_to_dto(id: &ExerciseId) -> String {
    match id {
        ExerciseId::Builtin(id) => id.clone(),
        ExerciseId::Custom(id) => format!("{CUSTOM_EXERCISE_ID_PREFIX}{id}"),
    }
}

fn exercise_id_from_dto(text: String) -> Result<ExerciseId, MappingError> {
    let Some(number) = text.strip_prefix(CUSTOM_EXERCISE_ID_PREFIX) else {
        return Ok(ExerciseId::Builtin(text));
    };
    number
        .parse::<u32>()
        .ok()
        .filter(|id| id.to_string() == number)
        .map(ExerciseId::Custom)
        .ok_or_else(|| MappingError::InvalidExerciseId(text.clone()))
}

fn custom_exercise_to_dto(exercise: &CustomExercise) -> CustomExerciseV1 {
    let info = exercise.info();
    CustomExerciseV1 {
        id: exercise_id_to_dto(&exercise.exercise_id()),
        name: info.name().to_string(),
        category: info.category().into(),
        muscle_group: info.muscle_group().into(),
        equipment: info.equipment().into(),
        measure: info.measure().into(),
        archived: exercise.is_archived(),
    }
}

fn custom_exercise_from_dto(dto: CustomExerciseV1) -> Result<CustomExercise, MappingError> {
    let ExerciseId::Custom(id) = exercise_id_from_dto(dto.id.clone())? else {
        return Err(MappingError::InvalidExerciseId(dto.id));
    };
    let info = ExerciseInfo::new(
        NonEmptyText::new(dto.name)?,
        dto.category.into(),
        dto.muscle_group.into(),
        dto.equipment.into(),
        dto.measure.into(),
    )?;
    Ok(CustomExercise::restore(id, info, dto.archived))
}

fn session_to_dto(session: &Session) -> SessionV1 {
    let status = session.status();
    SessionV1 {
        id: session.id().get(),
        name: session.name().to_string(),
        origin: match session.origin() {
            SessionOrigin::Planned => SessionOriginV1::Planned,
            SessionOrigin::Adhoc => SessionOriginV1::Adhoc,
        },
        status: match status {
            SessionStatus::Planned => SessionStatusV1::Planned,
            SessionStatus::InProgress { .. } => SessionStatusV1::InProgress,
            SessionStatus::Done { .. } => SessionStatusV1::Done,
        },
        planned_for: session.planned_for(),
        started_at: status.started_at(),
        finished_at: status.finished_at(),
        notes: session.notes().map(NonEmptyText::to_string),
        exercises: session
            .entries()
            .iter()
            .map(|entry| SessionExerciseV1 {
                entry_id: entry.entry_id().get(),
                exercise_id: exercise_id_to_dto(entry.exercise_id()),
                sets: entry.sets().iter().map(set_to_dto).collect(),
            })
            .collect(),
    }
}

fn session_from_dto(dto: SessionV1) -> Result<Session, MappingError> {
    let status = status_from_dto(dto.id, dto.status, dto.started_at, dto.finished_at)?;
    let entries = dto
        .exercises
        .into_iter()
        .map(|entry| {
            Ok(SessionExercise::restore(
                EntryId::new(entry.entry_id),
                exercise_id_from_dto(entry.exercise_id)?,
                entry
                    .sets
                    .into_iter()
                    .map(set_from_dto)
                    .collect::<Result<_, _>>()?,
            ))
        })
        .collect::<Result<Vec<_>, MappingError>>()?;
    Ok(Session::restore(
        SessionId::new(dto.id),
        NonEmptyText::new(dto.name)?,
        match dto.origin {
            SessionOriginV1::Planned => SessionOrigin::Planned,
            SessionOriginV1::Adhoc => SessionOrigin::Adhoc,
        },
        dto.planned_for,
        status,
        dto.notes.and_then(NonEmptyText::optional),
        entries,
    )?)
}

fn status_from_dto(
    session_id: u32,
    status: SessionStatusV1,
    started_at: Option<DateTime<FixedOffset>>,
    finished_at: Option<DateTime<FixedOffset>>,
) -> Result<SessionStatus, MappingError> {
    match (status, started_at, finished_at) {
        (SessionStatusV1::Planned, None, None) => Ok(SessionStatus::Planned),
        (SessionStatusV1::InProgress, Some(started_at), None) => {
            Ok(SessionStatus::InProgress { started_at })
        }
        (SessionStatusV1::Done, Some(started_at), Some(finished_at)) => Ok(SessionStatus::Done {
            started_at,
            finished_at,
        }),
        _ => Err(MappingError::InconsistentSessionStatus(session_id)),
    }
}

fn set_to_dto(set: &WorkoutSet) -> WorkoutSetV1 {
    let warmup = set.is_warmup();
    let completed_at = set.completed_at();
    match *set.measure() {
        SetMeasure::WeightReps { weight, reps } => WorkoutSetV1::WeightReps {
            weight_g: weight.grams(),
            reps: reps.get(),
            warmup,
            completed_at,
        },
        SetMeasure::Reps { reps, load } => WorkoutSetV1::Reps {
            reps: reps.get(),
            load_g: load.map(Load::grams),
            warmup,
            completed_at,
        },
        SetMeasure::Duration { duration, load } => WorkoutSetV1::Duration {
            duration_s: duration.get(),
            load_g: load.map(Load::grams),
            warmup,
            completed_at,
        },
        SetMeasure::Cardio(cardio) => WorkoutSetV1::Cardio {
            duration_s: cardio.duration().get(),
            distance_m: cardio.distance().map(Meters::get),
            calories_kcal: cardio.calories().map(Kcal::get),
            level: cardio.level(),
            incline_permille: cardio.incline().map(Permille::get),
            warmup,
            completed_at,
        },
    }
}

fn set_from_dto(dto: WorkoutSetV1) -> Result<WorkoutSet, MappingError> {
    let (measure, warmup, completed_at) = match dto {
        WorkoutSetV1::WeightReps {
            weight_g,
            reps,
            warmup,
            completed_at,
        } => (
            SetMeasure::WeightReps {
                weight: Weight::from_grams(weight_g),
                reps: non_zero(reps)?,
            },
            warmup,
            completed_at,
        ),
        WorkoutSetV1::Reps {
            reps,
            load_g,
            warmup,
            completed_at,
        } => (
            SetMeasure::Reps {
                reps: non_zero(reps)?,
                load: optional_load(load_g)?,
            },
            warmup,
            completed_at,
        ),
        WorkoutSetV1::Duration {
            duration_s,
            load_g,
            warmup,
            completed_at,
        } => (
            SetMeasure::Duration {
                duration: Seconds::new(duration_s)?,
                load: optional_load(load_g)?,
            },
            warmup,
            completed_at,
        ),
        WorkoutSetV1::Cardio {
            duration_s,
            distance_m,
            calories_kcal,
            level,
            incline_permille,
            warmup,
            completed_at,
        } => (
            SetMeasure::Cardio(CardioMeasure::new(
                Seconds::new(duration_s)?,
                distance_m.map(Meters::new).transpose()?,
                calories_kcal.map(Kcal::new),
                level,
                incline_permille.map(Permille::new),
            )),
            warmup,
            completed_at,
        ),
    };
    Ok(WorkoutSet::new(measure, warmup, completed_at))
}

fn non_zero(value: u32) -> Result<NonZeroU32, MappingError> {
    NonZeroU32::new(value).ok_or(MappingError::ZeroValue)
}

fn optional_load(grams: Option<u32>) -> Result<Option<Load>, MappingError> {
    Ok(grams.map(Load::from_grams).transpose()?)
}

macro_rules! map_identical_variants {
    ($domain:ident <=> $dto:ident { $($variant:ident),+ $(,)? }) => {
        impl From<$domain> for $dto {
            fn from(value: $domain) -> Self {
                match value {
                    $($domain::$variant => Self::$variant,)+
                }
            }
        }

        impl From<$dto> for $domain {
            fn from(value: $dto) -> Self {
                match value {
                    $($dto::$variant => Self::$variant,)+
                }
            }
        }
    };
}

map_identical_variants!(Category <=> CategoryV1 { Strength, Mobility, Stretching, Cardio });

map_identical_variants!(MeasureKind <=> MeasureKindV1 { WeightReps, Reps, Duration, Cardio });

map_identical_variants!(MuscleGroup <=> MuscleGroupV1 {
    Chest, Back, LowerBack, Traps, Shoulders, Biceps, Triceps, Forearms, Abs, Obliques,
    Glutes, Quads, Hamstrings, Adductors, Abductors, Calves, Neck, FullBody,
});

map_identical_variants!(Equipment <=> EquipmentV1 {
    Barbell, EzBar, TrapBar, Dumbbell, Kettlebell, Machine, SmithMachine, Cable, Bodyweight,
    Band, MedicineBall, Plate, Treadmill, Bike, Rower, Elliptical, StairClimber, SkiErg,
    JumpRope, Other,
});
