use domain::{
    Category, Db, Equipment, ExerciseInfo, MeasureKind, MuscleGroup, NonEmptyText,
    builtin_exercises,
};

use super::error::CatalogError;
use crate::shared::search::normalize_for_search;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseForm {
    pub name: String,
    pub category: Category,
    pub muscle_group: MuscleGroup,
    pub equipment: Equipment,
    pub measure: MeasureKind,
}

impl ExerciseForm {
    pub fn blank() -> Self {
        Self {
            name: String::new(),
            category: Category::Strength,
            muscle_group: MuscleGroup::Chest,
            equipment: Equipment::Machine,
            measure: MeasureKind::WeightReps,
        }
    }

    pub fn of_custom_exercise(db: &Db, id: u32) -> Option<Self> {
        let info = db.custom_exercise(id)?.info();
        Some(Self {
            name: info.name().to_string(),
            category: info.category(),
            muscle_group: info.muscle_group(),
            equipment: info.equipment(),
            measure: info.measure(),
        })
    }

    // Keeps the form valid: a category change falls back to its first allowed measure.
    pub fn set_category(&mut self, category: Category) {
        self.category = category;
        if !category.allows(self.measure)
            && let Some(first_allowed) = category.allowed_measures().first()
        {
            self.measure = *first_allowed;
        }
    }

    pub fn allowed_measures(&self) -> Vec<MeasureKind> {
        self.category.allowed_measures()
    }

    pub(super) fn to_info(&self) -> Result<ExerciseInfo, CatalogError> {
        let name = NonEmptyText::new(self.name.as_str()).map_err(|_| CatalogError::EmptyName)?;
        ExerciseInfo::new(
            name,
            self.category,
            self.muscle_group,
            self.equipment,
            self.measure,
        )
        .map_err(CatalogError::MeasureNotAllowed)
    }
}

// Names are compared without case or accents, against the pool and every custom exercise.
pub(super) fn ensure_unique_name(
    db: &Db,
    info: &ExerciseInfo,
    renamed_exercise: Option<u32>,
) -> Result<(), CatalogError> {
    let wanted = normalize_for_search(info.name().as_str());
    let builtin_names = builtin_exercises()
        .iter()
        .map(|exercise| exercise.info().name().as_str());
    let custom_names = db
        .custom_exercises()
        .iter()
        .filter(|exercise| Some(exercise.id()) != renamed_exercise)
        .map(|exercise| exercise.info().name().as_str());
    if builtin_names
        .chain(custom_names)
        .any(|name| normalize_for_search(name) == wanted)
    {
        Err(CatalogError::DuplicateName)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{custom_info, text};

    #[test]
    fn category_change_keeps_an_allowed_measure() {
        let mut form = ExerciseForm::blank();
        form.set_category(Category::Stretching);
        assert_eq!(form.measure, MeasureKind::Duration);
        form.set_category(Category::Strength);
        assert_eq!(form.measure, MeasureKind::Duration);
        assert_eq!(form.allowed_measures().len(), 3);
    }

    #[test]
    fn builds_info_or_reports_empty_name() {
        let mut form = ExerciseForm::blank();
        assert_eq!(form.to_info(), Err(CatalogError::EmptyName));
        form.name = " Rowing machine ".to_owned();
        assert_eq!(form.to_info().unwrap().name().as_str(), "Rowing machine");
    }

    #[test]
    fn reads_an_existing_custom_exercise() {
        let mut db = Db::default();
        db.add_custom_exercise(custom_info("Rowing", MeasureKind::Reps))
            .unwrap();
        let form = ExerciseForm::of_custom_exercise(&db, 1).unwrap();
        assert_eq!(form.name, "Rowing");
        assert_eq!(form.measure, MeasureKind::Reps);
        assert_eq!(ExerciseForm::of_custom_exercise(&db, 2), None);
    }

    #[test]
    fn names_must_be_unique_ignoring_case_and_accents() {
        let mut db = Db::default();
        db.add_custom_exercise(custom_info("Rowing", MeasureKind::WeightReps))
            .unwrap();
        let info = |name: &str| {
            ExerciseInfo::new(
                text(name),
                Category::Strength,
                MuscleGroup::Chest,
                Equipment::Barbell,
                MeasureKind::WeightReps,
            )
            .unwrap()
        };
        assert_eq!(
            ensure_unique_name(&db, &info("developpe COUCHE barre"), None),
            Err(CatalogError::DuplicateName)
        );
        assert_eq!(
            ensure_unique_name(&db, &info("rowing"), None),
            Err(CatalogError::DuplicateName)
        );
        assert_eq!(ensure_unique_name(&db, &info("rowing"), Some(1)), Ok(()));
        assert_eq!(
            ensure_unique_name(&db, &info("Squat Technogym"), None),
            Ok(())
        );
    }
}
