use scraps_libs::lang::LangCode;
use serde::de::{self, Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug)]
pub struct LangCodeConfig(LangCode);

impl<'de> Deserialize<'de> for LangCodeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        LangCode::from_str(s.as_str())
            .map(LangCodeConfig)
            .map_err(de::Error::custom)
    }
}

impl LangCodeConfig {
    pub fn into_lang_code(self) -> LangCode {
        self.0.clone()
    }
}
