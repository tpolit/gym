use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyText(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextError {
    Empty,
}

impl NonEmptyText {
    pub fn new(text: impl Into<String>) -> Result<Self, TextError> {
        let text = text.into();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            Err(TextError::Empty)
        } else if trimmed.len() == text.len() {
            Ok(Self(text))
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }

    pub fn optional(text: impl Into<String>) -> Option<Self> {
        Self::new(text).ok()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NonEmptyText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_non_empty_text() {
        assert_eq!(NonEmptyText::new("Push A").unwrap().as_str(), "Push A");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(NonEmptyText::new("  Jambes \n").unwrap().as_str(), "Jambes");
    }

    #[test]
    fn rejects_empty_or_blank_text() {
        assert_eq!(NonEmptyText::new(""), Err(TextError::Empty));
        assert_eq!(NonEmptyText::new(" \t\n"), Err(TextError::Empty));
    }

    #[test]
    fn optional_treats_blank_as_absent() {
        assert_eq!(NonEmptyText::optional("   "), None);
        assert_eq!(
            NonEmptyText::optional(" Siège cran 4 ").map(|t| t.to_string()),
            Some("Siège cran 4".to_owned())
        );
    }
}
