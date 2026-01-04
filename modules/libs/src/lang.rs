use core::str;
use iso639_enum::{IsoCompat, Language};
use std::fmt;

#[derive(Debug, Clone)]
pub struct LangCode(Language);

impl std::str::FromStr for LangCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        Language::from_iso639_1(s)
            .map(LangCode)
            .map_err(|_| format!("Failed to parse language code '{s}': invalid iso639-1 code"))
    }
}

impl fmt::Display for LangCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iso639_1().unwrap_or_else(|| self.0.name()))
    }
}

impl Default for LangCode {
    fn default() -> Self {
        LangCode(Language::Eng) // English
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::english("en", "en")]
    #[case::japanese("ja", "ja")]
    fn test_from_str_valid_language_codes(#[case] input: &str, #[case] expected: &str) {
        let lang = input.parse::<LangCode>().unwrap();
        assert_eq!(lang.to_string(), expected);
    }

    #[rstest]
    #[case::invalid_name("invalid")]
    #[case::invalid_code("zz")]
    #[case::empty("")]
    fn test_from_str_invalid_language_codes(#[case] input: &str) {
        let result = input.parse::<LangCode>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains(&format!("Failed to parse language code '{}'", input)));
    }

    #[rstest]
    #[case::english(Language::Eng, "en")]
    #[case::japanese(Language::Jpn, "ja")]
    fn test_display_formatting(#[case] lang: Language, #[case] expected: &str) {
        let lang_code = LangCode(lang);
        assert_eq!(format!("{}", lang_code), expected);
    }
}
