use crate::cli::path_resolver::PathResolver;
use crate::error::{anyhow::Context, CliError, ScrapsResult};
use crate::usecase::build::model::{
    build_config::BuildConfig, color_scheme::ColorScheme, css::CssMetadata, html::HtmlMetadata,
    list_view_configs::ListViewConfigs, paging::Paging, sort::SortKey,
};
use chrono_tz::Tz;
use config::Config;
use scraps_libs::lang::LangCode;
use scraps_libs::model::base_url::BaseUrl;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use url::Url;

use super::{
    base_url::BaseUrlConfig, color_scheme::ColorSchemeConfig, lang::LangCodeConfig,
    sort_key::SortKeyConfig,
};

/// SSG-specific configuration (site generation settings)
#[derive(Debug, Deserialize)]
pub struct SsgConfig {
    pub base_url: BaseUrlConfig,
    pub title: String,
    pub lang_code: Option<LangCodeConfig>,
    pub description: Option<String>,
    pub favicon: Option<Url>,
    pub build_search_index: Option<bool>,
    pub sort_key: Option<SortKeyConfig>,
    pub paginate_by: Option<usize>,
    pub color_scheme: Option<ColorSchemeConfig>,
}

impl SsgConfig {
    /// Gets base_url as BaseUrl
    pub fn base_url(&self) -> BaseUrl {
        self.base_url.clone().into_base_url()
    }
}

/// Lint-specific configuration. Each rule lives in its own nested table.
///
/// Rules surface opt-in/opt-out via `enabled` (default `true` when the
/// section is present) and rule-specific parameters as sibling fields. This
/// keeps selection and parameters co-located, so writing `[lint.stale_by_git]`
/// is enough to opt in with defaults.
#[derive(Debug, Deserialize, Default)]
pub struct LintConfig {
    pub stale_by_git: Option<StaleByGitConfig>,
}

/// Configuration for the `stale_by_git` lint rule.
///
/// `enabled` defaults to `true` when the section is present in `.scraps.toml`,
/// so a bare `[lint.stale_by_git]` opts the rule in. Setting `enabled = false`
/// disables the rule while preserving the threshold for later toggling.
#[derive(Debug, Deserialize)]
pub struct StaleByGitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub threshold_days: Option<u64>,
}

fn default_true() -> bool {
    true
}

/// Main configuration struct
#[derive(Debug, Deserialize)]
pub struct ScrapConfig {
    pub output_dir: Option<PathBuf>,
    pub timezone: Option<Tz>,
    pub ssg: Option<SsgConfig>,
    pub lint: Option<LintConfig>,
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
            .context(CliError::InvalidConfigFormat)
    }

    /// Requires SSG section to be present (for build/serve commands)
    pub fn require_ssg(&self) -> ScrapsResult<&SsgConfig> {
        self.ssg
            .as_ref()
            .ok_or_else(|| CliError::MissingSsgSection.into())
    }

    /// Gets optional base_url if ssg section is present
    pub fn get_base_url(&self) -> Option<BaseUrl> {
        self.ssg.as_ref().map(|s| s.base_url())
    }

    /// Resolve the settings a build run needs, applying the defaults for every
    /// omitted `[ssg]` key. `base_url` overrides the configured one, which
    /// `serve` needs so links point at the local address it listens on.
    pub fn to_build_config(&self, base_url: Option<BaseUrl>) -> ScrapsResult<BuildConfig> {
        let ssg = self.require_ssg()?;

        let default_lang_code = LangCode::default();
        let lang_code = ssg
            .lang_code
            .as_ref()
            .map(|c| c.as_lang_code())
            .unwrap_or(&default_lang_code);
        let html_metadata =
            HtmlMetadata::new(lang_code, &ssg.title, &ssg.description, &ssg.favicon);

        let default_color_scheme = ColorScheme::OsSetting;
        let css_metadata = CssMetadata::new(
            ssg.color_scheme
                .as_ref()
                .map(|c| c.as_color_scheme())
                .unwrap_or(&default_color_scheme),
        );

        let default_sort_key = SortKey::CommittedDate;
        let sort_key = ssg
            .sort_key
            .as_ref()
            .map(|s| s.as_sort_key())
            .unwrap_or(&default_sort_key);
        let paging = match ssg.paginate_by {
            None => Paging::Not,
            Some(u) => Paging::By(u),
        };
        let list_view_configs =
            ListViewConfigs::new(&ssg.build_search_index.unwrap_or(true), sort_key, &paging);

        Ok(BuildConfig {
            base_url: base_url.unwrap_or_else(|| ssg.base_url()),
            timezone: self.timezone.unwrap_or(chrono_tz::UTC),
            html_metadata,
            css_metadata,
            list_view_configs,
        })
    }
}
