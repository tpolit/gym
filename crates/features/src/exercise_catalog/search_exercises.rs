use domain::{Category, Db, Equipment, ExerciseId, ExerciseInfo, MuscleGroup};

use crate::shared::labels::{equipment_label, muscle_group_label};
use crate::shared::muscle_filter::matches_muscle;
use crate::shared::search::{matches_search, normalize_for_search};
use crate::shared::shown_builtin_exercises::shown_builtin_exercises;

// Families shown in the catalog are filters combining category and equipment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    GymStrength,
    Bodyweight,
    Mobility,
    Stretching,
    Cardio,
}

impl Family {
    pub const ALL: [Self; 5] = [
        Self::GymStrength,
        Self::Bodyweight,
        Self::Mobility,
        Self::Stretching,
        Self::Cardio,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::GymStrength => "Musculation",
            Self::Bodyweight => "Poids du corps",
            Self::Mobility => "Mobilité",
            Self::Stretching => "Étirements",
            Self::Cardio => "Cardio",
        }
    }

    fn contains(self, info: &ExerciseInfo) -> bool {
        let bodyweight = info.equipment() == Equipment::Bodyweight;
        match (self, info.category()) {
            (Self::GymStrength, Category::Strength) => !bodyweight,
            (Self::Bodyweight, Category::Strength) => bodyweight,
            (Self::Mobility, Category::Mobility)
            | (Self::Stretching, Category::Stretching)
            | (Self::Cardio, Category::Cardio) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogFilter {
    pub query: String,
    pub family: Option<Family>,
    pub muscle_group: Option<MuscleGroup>,
    pub show_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogItem {
    pub id: ExerciseId,
    pub name: String,
    pub detail: String,
    pub is_custom: bool,
    pub is_archived: bool,
}

pub fn search_exercises(db: &Db, filter: &CatalogFilter) -> Vec<CatalogItem> {
    let builtin = shown_builtin_exercises().map(|exercise| {
        let mut terms = vec![exercise.info().name().to_string()];
        terms.extend(exercise.aliases().iter().map(|alias| (*alias).to_owned()));
        (exercise.exercise_id(), exercise.info(), terms, false)
    });
    let custom = db
        .custom_exercises()
        .iter()
        .filter(|exercise| filter.show_archived || !exercise.is_archived())
        .map(|exercise| {
            let terms = vec![exercise.info().name().to_string()];
            (
                exercise.exercise_id(),
                exercise.info(),
                terms,
                exercise.is_archived(),
            )
        });
    let mut items: Vec<CatalogItem> = builtin
        .chain(custom)
        .filter(|(_, info, terms, _)| {
            filter.family.is_none_or(|family| family.contains(info))
                && matches_muscle(filter.muscle_group, info.muscle_group())
                && matches_search(&filter.query, terms)
        })
        .map(|(id, info, _, is_archived)| CatalogItem {
            is_custom: matches!(id, ExerciseId::Custom(_)),
            id,
            name: info.name().to_string(),
            detail: format!(
                "{} · {}",
                muscle_group_label(info.muscle_group()),
                equipment_label(info.equipment())
            ),
            is_archived,
        })
        .collect();
    items.sort_by_cached_key(|item| normalize_for_search(&item.name));
    items
}

#[cfg(test)]
mod tests {
    use domain::{MeasureKind, NonEmptyText};

    use super::*;
    use crate::test_support::{bench, custom_info, pull_up};

    fn ids(items: &[CatalogItem]) -> Vec<ExerciseId> {
        items.iter().map(|item| item.id.clone()).collect()
    }

    #[test]
    fn filters_by_family() {
        let db = Db::default();
        let gym = search_exercises(
            &db,
            &CatalogFilter {
                family: Some(Family::GymStrength),
                ..CatalogFilter::default()
            },
        );
        assert!(ids(&gym).contains(&bench()));
        assert!(!ids(&gym).contains(&pull_up()));
        let bodyweight = search_exercises(
            &db,
            &CatalogFilter {
                family: Some(Family::Bodyweight),
                ..CatalogFilter::default()
            },
        );
        assert!(ids(&bodyweight).contains(&pull_up()));
    }

    #[test]
    fn filters_by_main_muscle_group_combined_with_family_and_text() {
        let mut db = Db::default();
        let custom = db
            .add_custom_exercise(custom_info("Tirage Technogym", MeasureKind::WeightReps))
            .unwrap();
        let search = |family: Option<Family>, muscle_group: MuscleGroup, query: &str| {
            search_exercises(
                &db,
                &CatalogFilter {
                    query: query.to_owned(),
                    family,
                    muscle_group: Some(muscle_group),
                    ..CatalogFilter::default()
                },
            )
        };

        let chest = search(None, MuscleGroup::Chest, "");
        assert!(ids(&chest).contains(&bench()));
        assert!(
            chest
                .iter()
                .all(|item| item.detail.starts_with("Pectoraux · "))
        );

        let back = ids(&search(None, MuscleGroup::Back, ""));
        assert!(back.contains(&pull_up()) && back.contains(&custom));
        assert!(!back.contains(&bench()));

        let bodyweight_back = ids(&search(Some(Family::Bodyweight), MuscleGroup::Back, ""));
        assert!(bodyweight_back.contains(&pull_up()) && !bodyweight_back.contains(&custom));

        assert_eq!(ids(&search(None, MuscleGroup::Back, "technogym")), [custom]);
        assert!(search(None, MuscleGroup::Chest, "technogym").is_empty());
    }

    #[test]
    fn searches_names_and_aliases_without_accents() {
        let items = search_exercises(
            &Db::default(),
            &CatalogFilter {
                query: "bench COUCHÉ barre".to_owned(),
                ..CatalogFilter::default()
            },
        );
        assert_eq!(ids(&items), [bench()]);
        assert_eq!(items[0].detail, "Pectoraux · Barre");
    }

    #[test]
    fn finds_mobility_exercises_by_their_english_routine_names() {
        let found = |query: &str| {
            ids(&search_exercises(
                &Db::default(),
                &CatalogFilter {
                    query: query.to_owned(),
                    family: Some(Family::Mobility),
                    ..CatalogFilter::default()
                },
            ))
        };
        let builtin = |id: &str| ExerciseId::Builtin(id.to_owned());
        assert_eq!(
            found("alternate knees to floor"),
            [builtin("half_kneeling_hip_flexor_rock")]
        );
        assert_eq!(
            found("dynamic wipers"),
            [builtin("supine_windshield_wipers")]
        );
        assert_eq!(found("seated twist"), [builtin("pilates_spine_twist")]);
        assert!(found("rouleau").is_empty());
    }

    #[test]
    fn hides_archived_exercises_unless_asked() {
        let mut db = Db::default();
        let id = db
            .add_custom_exercise(custom_info("Tirage Technogym", MeasureKind::WeightReps))
            .unwrap();
        db.plan_session(
            NonEmptyText::new("Dos").unwrap(),
            None,
            std::slice::from_ref(&id),
        )
        .unwrap();
        db.remove_custom_exercise(1).unwrap();
        let query = CatalogFilter {
            query: "technogym".to_owned(),
            ..CatalogFilter::default()
        };
        assert!(search_exercises(&db, &query).is_empty());
        let with_archived = search_exercises(
            &db,
            &CatalogFilter {
                show_archived: true,
                ..query
            },
        );
        assert_eq!(ids(&with_archived), [id]);
        assert!(with_archived[0].is_custom && with_archived[0].is_archived);
    }

    #[test]
    fn lists_everything_sorted_by_default() {
        let items = search_exercises(&Db::default(), &CatalogFilter::default());
        assert_eq!(items.len(), shown_builtin_exercises().count());
        let names: Vec<String> = items
            .iter()
            .map(|i| normalize_for_search(&i.name))
            .collect();
        assert!(names.is_sorted());
    }
}
