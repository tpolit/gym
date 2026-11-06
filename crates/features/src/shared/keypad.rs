use crate::shared::decimal::{format_scaled, parse_scaled};

const MAX_LENGTH: usize = 7;

// Text typed on the custom keypad. The first key replaces the value shown
// on opening, like amount keypads in banking apps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeypadText {
    text: String,
    decimals: u32,
    replace_on_next_key: bool,
}

impl KeypadText {
    pub fn new(initial: impl Into<String>, decimals: u32) -> Self {
        Self {
            text: initial.into(),
            decimals,
            replace_on_next_key: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub const fn allows_decimal_separator(&self) -> bool {
        self.decimals > 0
    }

    pub fn press_digit(&mut self, digit: char) {
        if !digit.is_ascii_digit() {
            return;
        }
        self.start_typing();
        let fraction_is_full = self
            .text
            .split_once(',')
            .is_some_and(|(_, fraction)| fraction.len() >= self.decimals as usize);
        if fraction_is_full || self.text.len() >= MAX_LENGTH {
            return;
        }
        if self.text == "0" {
            self.text.clear();
        }
        self.text.push(digit);
    }

    pub fn press_decimal_separator(&mut self) {
        if !self.allows_decimal_separator() {
            return;
        }
        self.start_typing();
        if self.text.contains(',') || self.text.len() >= MAX_LENGTH {
            return;
        }
        if self.text.is_empty() {
            self.text.push('0');
        }
        self.text.push(',');
    }

    // Shortcut keys (−2,5, +0,5…): delta in the sub-unit, never below zero.
    pub fn add(&mut self, delta: i64) {
        let current = parse_scaled(&self.text, self.decimals).unwrap_or(0);
        let updated = (i64::from(current) + delta).clamp(0, i64::from(u32::MAX));
        let updated = u32::try_from(updated).unwrap_or(0);
        self.text = format_scaled(updated, self.decimals);
        self.replace_on_next_key = true;
    }

    pub fn press_delete(&mut self) {
        self.replace_on_next_key = false;
        self.text.pop();
    }

    fn start_typing(&mut self) {
        if self.replace_on_next_key {
            self.text.clear();
            self.replace_on_next_key = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_keys(keypad: &mut KeypadText, keys: &str) {
        for key in keys.chars() {
            match key {
                ',' => keypad.press_decimal_separator(),
                '<' => keypad.press_delete(),
                digit => keypad.press_digit(digit),
            }
        }
    }

    #[test]
    fn first_key_replaces_initial_value() {
        let mut keypad = KeypadText::new("80", 3);
        type_keys(&mut keypad, "82,5");
        assert_eq!(keypad.text(), "82,5");
    }

    #[test]
    fn delete_edits_initial_value() {
        let mut keypad = KeypadText::new("80", 3);
        type_keys(&mut keypad, "<5");
        assert_eq!(keypad.text(), "85");
    }

    #[test]
    fn limits_decimals_and_separators() {
        let mut keypad = KeypadText::new("", 1);
        type_keys(&mut keypad, ",55,");
        assert_eq!(keypad.text(), "0,5");

        let mut integer_only = KeypadText::new("", 0);
        type_keys(&mut integer_only, "1,2");
        assert_eq!(integer_only.text(), "12");
    }

    #[test]
    fn shortcuts_adjust_the_value_without_going_negative() {
        let mut keypad = KeypadText::new("80", 3);
        keypad.add(2_500);
        assert_eq!(keypad.text(), "82,5");
        keypad.add(-500);
        assert_eq!(keypad.text(), "82");
        type_keys(&mut keypad, "5");
        assert_eq!(keypad.text(), "5");
        let mut empty = KeypadText::new("", 3);
        empty.add(-2_500);
        assert_eq!(empty.text(), "0");
    }

    #[test]
    fn avoids_leading_zeros_and_long_numbers() {
        let mut keypad = KeypadText::new("", 3);
        type_keys(&mut keypad, "0012345678");
        assert_eq!(keypad.text(), "1234567");
    }
}
