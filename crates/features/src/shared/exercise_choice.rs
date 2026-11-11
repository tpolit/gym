use domain::ExerciseId;

use crate::shared::search::matches_search;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExerciseChoice {
    pub id: ExerciseId,
    pub name: String,
    pub detail: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_name_or_alias() {
        let choice = ExerciseChoice {
            id: ExerciseId::Builtin("pull_up".to_owned()),
            name: "Tractions".to_owned(),
            detail: "Dos · Poids du corps".to_owned(),
            search_terms: vec!["Tractions".to_owned(), "pull-up".to_owned()],
        };
        assert!(choice.matches("traction"));
        assert!(choice.matches("PULL"));
        assert!(!choice.matches("dips"));
    }
}
