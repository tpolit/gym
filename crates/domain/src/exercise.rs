use crate::measure::MeasureKind;
use crate::text::NonEmptyText;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExerciseId {
    Builtin(String),
    Custom(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Strength,
    Mobility,
    Stretching,
    Cardio,
}

impl Category {
    pub const ALL: [Self; 4] = [
        Self::Strength,
        Self::Mobility,
        Self::Stretching,
        Self::Cardio,
    ];

    pub fn allowed_measures(self) -> Vec<MeasureKind> {
        MeasureKind::ALL
            .into_iter()
            .filter(|measure| self.allows(*measure))
            .collect()
    }

    pub const fn allows(self, measure: MeasureKind) -> bool {
        matches!(
            (self, measure),
            (
                Self::Strength,
                MeasureKind::WeightReps | MeasureKind::Reps | MeasureKind::Duration
            ) | (Self::Mobility, MeasureKind::Reps | MeasureKind::Duration)
                | (Self::Stretching, MeasureKind::Duration)
                | (Self::Cardio, MeasureKind::Cardio)
        )
    }
}

// Persisted values (custom exercises): add new ones, never remove or rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MuscleGroup {
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

impl MuscleGroup {
    pub const ALL: [Self; 18] = [
        Self::Chest,
        Self::Back,
        Self::LowerBack,
        Self::Traps,
        Self::Shoulders,
        Self::Biceps,
        Self::Triceps,
        Self::Forearms,
        Self::Abs,
        Self::Obliques,
        Self::Glutes,
        Self::Quads,
        Self::Hamstrings,
        Self::Adductors,
        Self::Abductors,
        Self::Calves,
        Self::Neck,
        Self::FullBody,
    ];
}

// Persisted values (custom exercises): add new ones, never remove or rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Equipment {
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

impl Equipment {
    pub const ALL: [Self; 20] = [
        Self::Barbell,
        Self::EzBar,
        Self::TrapBar,
        Self::Dumbbell,
        Self::Kettlebell,
        Self::Machine,
        Self::SmithMachine,
        Self::Cable,
        Self::Bodyweight,
        Self::Band,
        Self::MedicineBall,
        Self::Plate,
        Self::Treadmill,
        Self::Bike,
        Self::Rower,
        Self::Elliptical,
        Self::StairClimber,
        Self::SkiErg,
        Self::JumpRope,
        Self::Other,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseInfoError {
    MeasureNotAllowed {
        category: Category,
        measure: MeasureKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseInfo {
    name: NonEmptyText,
    category: Category,
    muscle_group: MuscleGroup,
    equipment: Equipment,
    measure: MeasureKind,
}

impl ExerciseInfo {
    pub fn new(
        name: NonEmptyText,
        category: Category,
        muscle_group: MuscleGroup,
        equipment: Equipment,
        measure: MeasureKind,
    ) -> Result<Self, ExerciseInfoError> {
        if !category.allows(measure) {
            return Err(ExerciseInfoError::MeasureNotAllowed { category, measure });
        }
        Ok(Self {
            name,
            category,
            muscle_group,
            equipment,
            measure,
        })
    }

    pub fn name(&self) -> &NonEmptyText {
        &self.name
    }

    pub const fn category(&self) -> Category {
        self.category
    }

    pub const fn muscle_group(&self) -> MuscleGroup {
        self.muscle_group
    }

    pub const fn equipment(&self) -> Equipment {
        self.equipment
    }

    pub const fn measure(&self) -> MeasureKind {
        self.measure
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomExercise {
    id: u32,
    info: ExerciseInfo,
    archived: bool,
}

impl CustomExercise {
    pub const fn restore(id: u32, info: ExerciseInfo, archived: bool) -> Self {
        Self { id, info, archived }
    }

    pub const fn id(&self) -> u32 {
        self.id
    }

    pub const fn exercise_id(&self) -> ExerciseId {
        ExerciseId::Custom(self.id)
    }

    pub const fn info(&self) -> &ExerciseInfo {
        &self.info
    }

    pub const fn is_archived(&self) -> bool {
        self.archived
    }

    pub(crate) fn set_info(&mut self, info: ExerciseInfo) {
        self.info = info;
    }

    pub(crate) fn set_archived(&mut self, archived: bool) {
        self.archived = archived;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseSettings(NonEmptyText);

impl ExerciseSettings {
    pub const fn new(text: NonEmptyText) -> Self {
        Self(text)
    }

    pub const fn text(&self) -> &NonEmptyText {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name() -> NonEmptyText {
        NonEmptyText::new("Exercice").unwrap()
    }

    #[test]
    fn allowed_combinations_match_the_specification() {
        use MeasureKind::*;
        let allowed = [
            (Category::Strength, [true, true, true, false]),
            (Category::Mobility, [false, true, true, false]),
            (Category::Stretching, [false, false, true, false]),
            (Category::Cardio, [false, false, false, true]),
        ];
        for (category, expected) in allowed {
            let actual = [WeightReps, Reps, Duration, Cardio].map(|m| category.allows(m));
            assert_eq!(actual, expected, "{category:?}");
        }
    }

    #[test]
    fn value_lists_have_no_duplicates() {
        use std::collections::BTreeSet;
        assert_eq!(BTreeSet::from(Category::ALL).len(), Category::ALL.len());
        assert_eq!(
            BTreeSet::from(MuscleGroup::ALL).len(),
            MuscleGroup::ALL.len()
        );
        assert_eq!(BTreeSet::from(Equipment::ALL).len(), Equipment::ALL.len());
        assert_eq!(
            BTreeSet::from(MeasureKind::ALL).len(),
            MeasureKind::ALL.len()
        );
    }

    #[test]
    fn allowed_measures_follow_allows() {
        assert_eq!(
            Category::Mobility.allowed_measures(),
            [MeasureKind::Reps, MeasureKind::Duration]
        );
        assert_eq!(Category::Cardio.allowed_measures(), [MeasureKind::Cardio]);
    }

    #[test]
    fn info_accepts_allowed_combination() {
        let info = ExerciseInfo::new(
            name(),
            Category::Strength,
            MuscleGroup::Back,
            Equipment::Bodyweight,
            MeasureKind::Reps,
        )
        .unwrap();
        assert_eq!(info.name().as_str(), "Exercice");
        assert_eq!(info.category(), Category::Strength);
        assert_eq!(info.muscle_group(), MuscleGroup::Back);
        assert_eq!(info.equipment(), Equipment::Bodyweight);
        assert_eq!(info.measure(), MeasureKind::Reps);
    }

    #[test]
    fn info_rejects_forbidden_combination() {
        let result = ExerciseInfo::new(
            name(),
            Category::Stretching,
            MuscleGroup::Hamstrings,
            Equipment::Bodyweight,
            MeasureKind::WeightReps,
        );
        assert_eq!(
            result,
            Err(ExerciseInfoError::MeasureNotAllowed {
                category: Category::Stretching,
                measure: MeasureKind::WeightReps,
            })
        );
    }

    #[test]
    fn custom_exercise_has_custom_id() {
        let info = ExerciseInfo::new(
            name(),
            Category::Strength,
            MuscleGroup::Chest,
            Equipment::Machine,
            MeasureKind::WeightReps,
        )
        .unwrap();
        let exercise = CustomExercise::restore(3, info, false);
        assert_eq!(exercise.exercise_id(), ExerciseId::Custom(3));
        assert!(!exercise.is_archived());
    }
}
