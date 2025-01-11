use crate::build::model::sort::SortKey;
use chrono_tz::Tz;
use config::Config;
use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapConfig {
    pub base_url: Url,
    pub title: String,
    pub description: Option<String>,
    pub favicon: Option<Url>,
    pub timezone: Option<Tz>,
    pub build_search_index: Option<bool>,
    pub sort_key: Option<SortKeyConfig>,
    pub paginate_by: Option<usize>,
}

impl ScrapConfig {
    pub fn new() -> ScrapResult<ScrapConfig> {
        let config = Config::builder()
            .add_source(config::File::with_name("Config.toml"))
            .build()
            .context(ScrapError::ConfigLoad)?;
        config
            .try_deserialize::<ScrapConfig>()
            .context(ScrapError::ConfigLoad)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "SortKey", rename_all = "snake_case")]
pub enum SerdeSortKey {
    CommittedDate,
    LinkedCount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SortKeyConfig(#[serde(with = "SerdeSortKey")] SortKey);

impl SortKeyConfig {
    pub fn into_sort_key(self) -> SortKey {
        self.0.clone()
    }
}
