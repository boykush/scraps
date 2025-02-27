use super::error::ScrapError;
use iso639_1::Iso639_1;

#[derive(Debug, Clone)]
pub struct LangCode(Iso639_1);

impl std::str::FromStr for LangCode {
    type Err = ScrapError;

    fn from_str(s: &str) -> Result<Self, ScrapError> {
        Iso639_1::try_from(s)
            .map(|i| LangCode(i))
            .map_err(|_| ScrapError::FromStrErr)
    }
}

impl ToString for LangCode {
    fn to_string(&self) -> String {
        self.0.name().to_string()
    }
}

impl Default for LangCode {
    fn default() -> Self {
        LangCode(Iso639_1::En)
    }
}
