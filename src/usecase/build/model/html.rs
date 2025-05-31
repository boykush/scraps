use scraps_lang::LangCode;
use url::Url;

#[derive(Clone)]
pub struct HtmlMetadata {
    lang_code: LangCode,
    title: String,
    description: Option<String>,
    favicon: Option<Url>,
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
}
