// Schema v1. Frozen once published: any change requires a new DTO
// and a migration. The only changes allowed without a new version are
// compatible additions: optional field with `#[serde(default)]`, new enum value.

use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbV1 {
    pub schema_version: u32,
    pub preferences: PreferencesV1,
    pub custom_exercises: Vec<CustomExerciseV1>,
    pub exercise_settings: BTreeMap<String, String>,
    pub sessions: Vec<SessionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreferencesV1 {
    pub weekly_goal: Option<u8>,
    // Added after v1 (compatible): absent from older blobs, omitted when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    // Added after v1 (compatible): absent from older blobs, omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_weights: Vec<BodyWeightV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyWeightV1 {
    pub date: NaiveDate,
    pub weight_g: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomExerciseV1 {
    pub id: String,
    pub name: String,
    pub category: CategoryV1,
    pub muscle_group: MuscleGroupV1,
    pub equipment: EquipmentV1,
    pub measure: MeasureKindV1,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionV1 {
    pub id: u32,
    pub name: String,
    pub origin: SessionOriginV1,
    pub status: SessionStatusV1,
    pub planned_for: Option<NaiveDate>,
    pub started_at: Option<DateTime<FixedOffset>>,
    pub finished_at: Option<DateTime<FixedOffset>>,
    pub notes: Option<String>,
    pub exercises: Vec<SessionExerciseV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionExerciseV1 {
    pub entry_id: u32,
    pub exercise_id: String,
    pub sets: Vec<WorkoutSetV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkoutSetV1 {
    WeightReps {
        weight_g: u32,
        reps: u32,
        warmup: bool,
        completed_at: Option<DateTime<FixedOffset>>,
    },
    Reps {
        reps: u32,
        load_g: Option<u32>,
        warmup: bool,
        completed_at: Option<DateTime<FixedOffset>>,
    },
    Duration {
        duration_s: u32,
        load_g: Option<u32>,
        warmup: bool,
        completed_at: Option<DateTime<FixedOffset>>,
    },
    Cardio {
        duration_s: u32,
        distance_m: Option<u32>,
        calories_kcal: Option<u32>,
        level: Option<u32>,
        incline_permille: Option<u32>,
        warmup: bool,
        completed_at: Option<DateTime<FixedOffset>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOriginV1 {
    Planned,
    Adhoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatusV1 {
    Planned,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoryV1 {
    Strength,
    Mobility,
    Stretching,
    Cardio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureKindV1 {
    WeightReps,
    Reps,
    Duration,
    Cardio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MuscleGroupV1 {
    Chest,
    Back,
    LowerBack,
    Traps,
    Shoulders,
    Biceps,
    Triceps,
    Forearms,
    Abs,
    Obliques,
    Glutes,
    Quads,
    Hamstrings,
    Adductors,
    Abductors,
    Calves,
    Neck,
    FullBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentV1 {
    Barbell,
    EzBar,
    TrapBar,
    Dumbbell,
    Kettlebell,
    Machine,
    SmithMachine,
    Cable,
    Bodyweight,
    Band,
    MedicineBall,
    Plate,
    Treadmill,
    Bike,
    Rower,
    Elliptical,
    StairClimber,
    SkiErg,
    JumpRope,
    Other,
}
