use scraps_libs::model::base_url::BaseUrl;
use serde::de::{self, Deserialize, Deserializer};
use url::Url;

#[derive(Debug, Clone)]
pub struct BaseUrlConfig(BaseUrl);

impl<'de> Deserialize<'de> for BaseUrlConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url = Url::deserialize(deserializer)?;
        BaseUrl::new(url)
            .map(BaseUrlConfig)
            .map_err(de::Error::custom)
    }
}

impl BaseUrlConfig {
    pub fn into_base_url(self) -> BaseUrl {
        self.0
    }
}
