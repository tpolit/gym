// Integer values stored in a sub-unit (grams, meters, per mille), displayed and typed
// with a decimal comma in the usual unit: 82500 g <-> "82,5" kg.

pub fn format_scaled(value: u32, decimals: u32) -> String {
    let scale = 10_u32.pow(decimals);
    let integer_part = value / scale;
    let fraction = value % scale;
    if fraction == 0 {
        return integer_part.to_string();
    }
    let fraction_digits = format!("{fraction:0width$}", width = decimals as usize);
    format!("{integer_part},{}", fraction_digits.trim_end_matches('0'))
}

pub fn parse_scaled(text: &str, decimals: u32) -> Option<u32> {
    let normalized = text.trim().replace('.', ",");
    let (integer_text, fraction_text) = normalized
        .split_once(',')
        .unwrap_or((normalized.as_str(), ""));
    let all_digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
    if integer_text.is_empty() && fraction_text.is_empty()
        || !all_digits(integer_text)
        || !all_digits(fraction_text)
        || fraction_text.len() > decimals as usize
    {
        return None;
    }
    let integer_part: u32 = if integer_text.is_empty() {
        0
    } else {
        integer_text.parse().ok()?
    };
    let fraction: u32 = if fraction_text.is_empty() {
        0
    } else {
        fraction_text.parse().ok()?
    };
    let missing_decimals = decimals - fraction_text.len() as u32;
    integer_part
        .checked_mul(10_u32.pow(decimals))?
        .checked_add(fraction.checked_mul(10_u32.pow(missing_decimals))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_french_decimal_comma() {
        assert_eq!(format_scaled(82_500, 3), "82,5");
        assert_eq!(format_scaled(80_000, 3), "80");
        assert_eq!(format_scaled(82_250, 3), "82,25");
        assert_eq!(format_scaled(500, 3), "0,5");
        assert_eq!(format_scaled(25, 1), "2,5");
        assert_eq!(format_scaled(12, 0), "12");
    }

    #[test]
    fn parses_comma_or_dot() {
        assert_eq!(parse_scaled("82,5", 3), Some(82_500));
        assert_eq!(parse_scaled(" 82.5 ", 3), Some(82_500));
        assert_eq!(parse_scaled("80", 3), Some(80_000));
        assert_eq!(parse_scaled("0,25", 3), Some(250));
        assert_eq!(parse_scaled(",5", 3), Some(500));
        assert_eq!(parse_scaled("80,", 3), Some(80_000));
        assert_eq!(parse_scaled("2,5", 1), Some(25));
        assert_eq!(parse_scaled("12", 0), Some(12));
    }

    #[test]
    fn rejects_invalid_text() {
        for text in ["", ",", "abc", "8a", "1,2,3", "-5", "1,2345"] {
            assert_eq!(parse_scaled(text, 3), None, "{text:?}");
        }
        assert_eq!(parse_scaled("1,5", 0), None);
        assert_eq!(parse_scaled("99999999", 3), None);
    }
}
