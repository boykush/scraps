use crate::error::{anyhow::Context, CliError, ScrapsResult};
use chrono_tz::Tz;
use config::Config;
use serde::Deserialize;
use url::Url;

use super::{color_scheme::ColorSchemeConfig, lang::LangCodeConfig, sort_key::SortKeyConfig};

#[derive(Debug, Deserialize)]
pub struct ScrapConfig {
    pub base_url: Url,
    pub lang_code: Option<LangCodeConfig>,
    pub title: String,
    pub description: Option<String>,
    pub favicon: Option<Url>,
    pub timezone: Option<Tz>,
    pub build_search_index: Option<bool>,
    pub sort_key: Option<SortKeyConfig>,
    pub paginate_by: Option<usize>,
    pub color_scheme: Option<ColorSchemeConfig>,
}

impl ScrapConfig {
    pub fn new() -> ScrapsResult<ScrapConfig> {
        let config = Config::builder()
            .add_source(config::File::with_name("Config.toml"))
            .build()
            .context(CliError::ConfigLoad)?;
        config
            .try_deserialize::<ScrapConfig>()
            .context(CliError::ConfigLoad)
    }
}
