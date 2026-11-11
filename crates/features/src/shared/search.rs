const COMBINING_DIACRITICS: std::ops::RangeInclusive<char> = '\u{300}'..='\u{36F}';

// A hand-written fold of Latin accents rather than Unicode normalization: the normalization
// tables weighed more than 100 KB in the .wasm, and exercise names are French.
pub fn normalize_for_search(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => normalized.push('a'),
            'ç' => normalized.push('c'),
            'è' | 'é' | 'ê' | 'ë' => normalized.push('e'),
            'ì' | 'í' | 'î' | 'ï' => normalized.push('i'),
            'ñ' => normalized.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => normalized.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => normalized.push('u'),
            'ý' | 'ÿ' => normalized.push('y'),
            'æ' => normalized.push_str("ae"),
            'œ' => normalized.push_str("oe"),
            'ß' => normalized.push_str("ss"),
            '’' => normalized.push('\''),
            c if COMBINING_DIACRITICS.contains(&c) => {}
            c => normalized.push(c),
        }
    }
    normalized
}

pub fn matches_search(query: &str, terms: &[String]) -> bool {
    let normalized_terms: Vec<String> = terms
        .iter()
        .map(|term| normalize_for_search(term))
        .collect();
    normalize_for_search(query)
        .split_whitespace()
        .all(|word| normalized_terms.iter().any(|term| term.contains(word)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terms(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn ignores_case_and_accents() {
        assert_eq!(
            normalize_for_search("Développé ÉLÉVATIONS"),
            "developpe elevations"
        );
        assert!(matches_search(
            "developpe",
            &terms(&["Développé couché barre"])
        ));
        assert!(matches_search("ÉLÉV", &terms(&["Élévations latérales"])));
    }

    #[test]
    fn folds_ligatures_apostrophes_and_decomposed_accents() {
        assert_eq!(normalize_for_search("Œuf Ÿ ß"), "oeuf y ss");
        assert_eq!(
            normalize_for_search("Vélo d’appartement"),
            "velo d'appartement"
        );
        assert_eq!(normalize_for_search("Developpe\u{301}"), "developpe");
        assert!(matches_search("pec deck", &terms(&["Pec deck"])));
    }

    #[test]
    fn every_word_must_match_one_of_the_terms() {
        let bench = terms(&["Développé couché barre", "bench press", "DC"]);
        assert!(matches_search("couché barre", &bench));
        assert!(matches_search("dc", &bench));
        assert!(matches_search("bench couché", &bench));
        assert!(!matches_search("couché haltères", &bench));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(matches_search("  ", &terms(&["Rameur"])));
    }
}
