use crate::cli::path_resolver::PathResolver;
use crate::error::{anyhow::Context, CliError, ScrapsResult};
use chrono_tz::Tz;
use config::Config;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use url::Url;

use super::{
    base_url::BaseUrlConfig, color_scheme::ColorSchemeConfig, lang::LangCodeConfig,
    sort_key::SortKeyConfig,
};

#[derive(Debug, Deserialize)]
pub struct ScrapConfig {
    pub base_url: BaseUrlConfig,
    pub lang_code: Option<LangCodeConfig>,
    pub scraps_dir: Option<PathBuf>,
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
    pub fn from_path(project_path: Option<&Path>) -> ScrapsResult<ScrapConfig> {
        let path_resolver = PathResolver::new(project_path)?;
        let config_path = path_resolver.config_path();

        let config = Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .context(CliError::ConfigLoad)?;

        config
            .try_deserialize::<ScrapConfig>()
            .context(CliError::ConfigLoad)
    }
}
