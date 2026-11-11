use domain::{ExerciseId, MuscleGroup};

use crate::shared::muscle_filter::matches_muscle;
use crate::shared::search::matches_search;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseChoice {
    pub id: ExerciseId,
    pub name: String,
    pub detail: String,
    pub muscle_group: MuscleGroup,
    pub search_terms: Vec<String>,
}

impl ExerciseChoice {
    pub fn matches(&self, query: &str) -> bool {
        matches_search(query, &self.search_terms)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExerciseChoices {
    pub recent: Vec<ExerciseChoice>,
    pub all: Vec<ExerciseChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceSection {
    pub title: &'static str,
    pub choices: Vec<ExerciseChoice>,
}

impl ExerciseChoices {
    // Empty sections are left out: no section at all means nothing was found.
    pub fn sections(&self, query: &str, muscle: Option<MuscleGroup>) -> Vec<ChoiceSection> {
        let section = |title, list: &[ExerciseChoice], query: &str| ChoiceSection {
            title,
            choices: list
                .iter()
                .filter(|choice| {
                    matches_muscle(muscle, choice.muscle_group) && choice.matches(query)
                })
                .cloned()
                .collect(),
        };
        let sections = if query.trim().is_empty() {
            vec![
                section("Récents", &self.recent, ""),
                section("Tous les exercices", &self.all, ""),
            ]
        } else {
            vec![section("Résultats", &self.all, query)]
        };
        sections
            .into_iter()
            .filter(|section| !section.choices.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(id: &str, name: &str, muscle_group: MuscleGroup) -> ExerciseChoice {
        ExerciseChoice {
            id: ExerciseId::Builtin(id.to_owned()),
            name: name.to_owned(),
            detail: String::new(),
            muscle_group,
            search_terms: vec![name.to_owned()],
        }
    }

    fn choices() -> ExerciseChoices {
        let bench = choice("bench", "Développé couché", MuscleGroup::Chest);
        let pull_up = choice("pull_up", "Tractions", MuscleGroup::Back);
        let fly = choice("fly", "Écarté couché", MuscleGroup::Chest);
        ExerciseChoices {
            recent: vec![pull_up.clone(), bench.clone()],
            all: vec![bench, fly, pull_up],
        }
    }

    fn summary(sections: &[ChoiceSection]) -> Vec<(&'static str, Vec<String>)> {
        sections
            .iter()
            .map(|section| {
                let names = section.choices.iter().map(|c| c.name.clone()).collect();
                (section.title, names)
            })
            .collect()
    }

    #[test]
    fn matches_name_or_alias() {
        let choice = ExerciseChoice {
            search_terms: vec!["Tractions".to_owned(), "pull-up".to_owned()],
            ..choice("pull_up", "Tractions", MuscleGroup::Back)
        };
        assert!(choice.matches("traction"));
        assert!(choice.matches("PULL"));
        assert!(!choice.matches("dips"));
    }

    #[test]
    fn without_query_shows_recent_then_all() {
        let sections = choices().sections(" ", None);
        assert_eq!(
            summary(&sections),
            [
                (
                    "Récents",
                    vec!["Tractions".into(), "Développé couché".into()]
                ),
                (
                    "Tous les exercices",
                    vec![
                        "Développé couché".into(),
                        "Écarté couché".into(),
                        "Tractions".into()
                    ]
                ),
            ]
        );
    }

    #[test]
    fn muscle_filter_applies_to_recent_and_all() {
        let sections = choices().sections("", Some(MuscleGroup::Chest));
        assert_eq!(
            summary(&sections),
            [
                ("Récents", vec!["Développé couché".into()]),
                (
                    "Tous les exercices",
                    vec!["Développé couché".into(), "Écarté couché".into()]
                ),
            ]
        );
    }

    #[test]
    fn query_shows_results_combined_with_the_muscle_filter() {
        let all = choices();
        assert_eq!(
            summary(&all.sections("couche", None)),
            [(
                "Résultats",
                vec!["Développé couché".into(), "Écarté couché".into()]
            )]
        );
        assert_eq!(
            summary(&all.sections("couche", Some(MuscleGroup::Back))),
            []
        );
    }

    #[test]
    fn empty_sections_are_left_out() {
        let sections = choices().sections("", Some(MuscleGroup::Calves));
        assert!(sections.is_empty());
        let no_recent = ExerciseChoices {
            recent: Vec::new(),
            ..choices()
        };
        assert_eq!(
            summary(&no_recent.sections("", Some(MuscleGroup::Back))),
            [("Tous les exercices", vec!["Tractions".into()])]
        );
    }
}
