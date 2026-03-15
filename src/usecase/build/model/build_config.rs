use chrono_tz::Tz;
use scraps_libs::model::base_url::BaseUrl;

use super::{css::CssMetadata, html::HtmlMetadata, list_view_configs::ListViewConfigs};

pub struct BuildConfig<'a> {
    pub base_url: &'a BaseUrl,
    pub timezone: Tz,
    pub html_metadata: &'a HtmlMetadata,
    pub css_metadata: &'a CssMetadata,
    pub list_view_configs: &'a ListViewConfigs,
}
