use core::str;
use iso639_enum::{Language, IsoCompat};
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
        write!(f, "{}", self.0.iso639_1().unwrap_or("unknown"))
    }
}

impl Default for LangCode {
    fn default() -> Self {
        LangCode(Language::Eng)  // English
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_valid_language_codes() {
        let lang = "en".parse::<LangCode>().unwrap();
        assert_eq!(lang.to_string(), "en");

        let lang = "ja".parse::<LangCode>().unwrap();
        assert_eq!(lang.to_string(), "ja");
    }

    #[test]
    fn test_from_str_invalid_language_codes() {
        let result = "invalid".parse::<LangCode>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to parse language code 'invalid'"));

        let result = "zz".parse::<LangCode>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to parse language code 'zz'"));

        let result = "".parse::<LangCode>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to parse language code ''"));
    }

    #[test]
    fn test_display_formatting() {
        let lang = LangCode(Language::Eng);
        assert_eq!(format!("{}", lang), "en");

        let lang = LangCode(Language::Jpn);
        assert_eq!(format!("{}", lang), "ja");
    }
}
