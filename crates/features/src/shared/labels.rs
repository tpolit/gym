use domain::{Category, Equipment, MeasureKind, MuscleGroup};

pub const fn category_label(category: Category) -> &'static str {
    match category {
        Category::Strength => "Musculation",
        Category::Mobility => "Mobilité",
        Category::Stretching => "Étirement",
        Category::Cardio => "Cardio",
    }
}

pub const fn measure_label(measure: MeasureKind) -> &'static str {
    match measure {
        MeasureKind::WeightReps => "Poids × répétitions",
        MeasureKind::Reps => "Répétitions (lest facultatif)",
        MeasureKind::Duration => "Durée (lest facultatif)",
        MeasureKind::Cardio => "Cardio (durée, distance…)",
    }
}

pub const fn muscle_group_label(group: MuscleGroup) -> &'static str {
    match group {
        MuscleGroup::Chest => "Pectoraux",
        MuscleGroup::Back => "Dos",
        MuscleGroup::LowerBack => "Lombaires",
        MuscleGroup::Traps => "Trapèzes",
        MuscleGroup::Shoulders => "Épaules",
        MuscleGroup::Biceps => "Biceps",
        MuscleGroup::Triceps => "Triceps",
        MuscleGroup::Forearms => "Avant-bras",
        MuscleGroup::Abs => "Abdominaux",
        MuscleGroup::Obliques => "Obliques",
        MuscleGroup::Glutes => "Fessiers",
        MuscleGroup::Quads => "Quadriceps",
        MuscleGroup::Hamstrings => "Ischio-jambiers",
        MuscleGroup::Adductors => "Adducteurs",
        MuscleGroup::Abductors => "Abducteurs",
        MuscleGroup::Calves => "Mollets",
        MuscleGroup::Neck => "Cou",
        MuscleGroup::FullBody => "Corps entier",
    }
}

pub const fn equipment_label(equipment: Equipment) -> &'static str {
    match equipment {
        Equipment::Barbell => "Barre",
        Equipment::EzBar => "Barre EZ",
        Equipment::TrapBar => "Trap bar",
        Equipment::Dumbbell => "Haltères",
        Equipment::Kettlebell => "Kettlebell",
        Equipment::Machine => "Machine",
        Equipment::SmithMachine => "Smith machine",
        Equipment::Cable => "Poulie",
        Equipment::Bodyweight => "Poids du corps",
        Equipment::Band => "Élastique",
        Equipment::MedicineBall => "Médecine-ball",
        Equipment::Plate => "Disque",
        Equipment::Treadmill => "Tapis",
        Equipment::Bike => "Vélo",
        Equipment::Rower => "Rameur",
        Equipment::Elliptical => "Elliptique",
        Equipment::StairClimber => "Stepper",
        Equipment::SkiErg => "SkiErg",
        Equipment::JumpRope => "Corde à sauter",
        Equipment::Other => "Autre",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_in_french() {
        assert_eq!(
            muscle_group_label(MuscleGroup::Hamstrings),
            "Ischio-jambiers"
        );
        assert_eq!(equipment_label(Equipment::Cable), "Poulie");
        assert_eq!(category_label(Category::Stretching), "Étirement");
        assert_eq!(
            measure_label(MeasureKind::WeightReps),
            "Poids × répétitions"
        );
    }
}
