use core::str;
use iso639_1::Iso639_1;
use std::fmt;

#[derive(Debug, Clone)]
pub struct LangCode(Iso639_1);

impl std::str::FromStr for LangCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        Iso639_1::try_from(s)
            .map(LangCode)
            .map_err(|e| format!("Failed to parse language code '{}': {}", s, e))
    }
}

impl fmt::Display for LangCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.name())
    }
}

impl Default for LangCode {
    fn default() -> Self {
        LangCode(Iso639_1::En)
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
        assert!(result.unwrap_err().contains("Failed to parse language code 'invalid'"));

        let result = "zz".parse::<LangCode>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse language code 'zz'"));

        let result = "".parse::<LangCode>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse language code ''"));
    }

    #[test]
    fn test_display_formatting() {
        let lang = LangCode(Iso639_1::En);
        assert_eq!(format!("{}", lang), "en");

        let lang = LangCode(Iso639_1::Ja);
        assert_eq!(format!("{}", lang), "ja");
    }
}
