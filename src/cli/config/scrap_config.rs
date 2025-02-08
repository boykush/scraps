use chrono_tz::Tz;
use config::Config;
use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};
use serde::Deserialize;
use url::Url;

use super::sort_key::SortKeyConfig;

#[derive(Debug, Deserialize)]
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