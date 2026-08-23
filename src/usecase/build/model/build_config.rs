use chrono_tz::Tz;
use scraps_libs::model::base_url::BaseUrl;

use super::{css::CssMetadata, html::HtmlMetadata, list_view_configs::ListViewConfigs};

/// Site-wide settings one build run needs. Grouped so `build` and `serve`
/// derive them the same way and hand the usecase a single value.
pub struct BuildConfig {
    pub base_url: BaseUrl,
    pub timezone: Tz,
    pub html_metadata: HtmlMetadata,
    pub css_metadata: CssMetadata,
    pub list_view_configs: ListViewConfigs,
}
