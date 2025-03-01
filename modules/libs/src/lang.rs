use super::error::ScrapError;
use iso639_1::Iso639_1;
use std::fmt;

#[derive(Debug, Clone)]
pub struct LangCode(Iso639_1);

impl std::str::FromStr for LangCode {
    type Err = ScrapError;

    fn from_str(s: &str) -> Result<Self, ScrapError> {
        Iso639_1::try_from(s)
            .map(LangCode)
            .map_err(|_| ScrapError::FromStrErr)
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
