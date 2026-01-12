use crate::cli::path_resolver::PathResolver;
use crate::error::{anyhow::Context, CliError, ScrapsResult};
use chrono_tz::Tz;
use config::Config;
use scraps_libs::model::base_url::BaseUrl;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use url::Url;

use super::{
    base_url::BaseUrlConfig, color_scheme::ColorSchemeConfig, lang::LangCodeConfig,
    sort_key::SortKeyConfig,
};

#[derive(Debug, Deserialize)]
pub struct ScrapConfig {
    pub base_url: Option<BaseUrlConfig>,
    pub title: Option<String>,
    pub scraps_dir: Option<PathBuf>,
    pub lang_code: Option<LangCodeConfig>,
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

    /// Validates that title is present and non-empty
    pub fn require_title(&self) -> ScrapsResult<String> {
        self.title
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                CliError::MissingRequiredConfig {
                    field: "title".to_string(),
                }
                .into()
            })
    }

    /// Validates that base_url is present
    pub fn require_base_url(&self) -> ScrapsResult<BaseUrl> {
        self.base_url
            .as_ref()
            .ok_or_else(|| {
                CliError::MissingRequiredConfig {
                    field: "base_url".to_string(),
                }
                .into()
            })
            .map(|config| config.clone().into_base_url())
    }

    /// Gets optional base_url if present
    pub fn get_base_url(&self) -> Option<BaseUrl> {
        self.base_url.as_ref().map(|c| c.clone().into_base_url())
    }
}
