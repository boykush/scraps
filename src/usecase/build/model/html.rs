use scraps_libs::lang::LangCode;
use url::Url;

use super::color_scheme::ColorScheme;

#[derive(Clone)]
pub struct HtmlMetadata {
    lang_code: LangCode,
    title: String,
    description: Option<String>,
    favicon: Option<Url>,
    color_scheme: ColorScheme,
}

impl HtmlMetadata {
    pub fn new(
        lang_code: &LangCode,
        title: &str,
        description: &Option<String>,
        favicon: &Option<Url>,
    ) -> HtmlMetadata {
        HtmlMetadata {
            lang_code: lang_code.clone(),
            title: title.to_string(),
            description: description.clone(),
            favicon: favicon.clone(),
            color_scheme: ColorScheme::OsSetting,
        }
    }

    /// The page names its scheme itself, so the browser paints the right
    /// ground before the stylesheet arrives.
    pub fn with_color_scheme(self, color_scheme: &ColorScheme) -> HtmlMetadata {
        HtmlMetadata {
            color_scheme: color_scheme.clone(),
            ..self
        }
    }

    pub fn lang_code(&self) -> LangCode {
        self.lang_code.clone()
    }

    pub fn title(&self) -> String {
        self.title.to_string()
    }
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }
    pub fn favicon(&self) -> Option<Url> {
        self.favicon.clone()
    }
    pub fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }
}
