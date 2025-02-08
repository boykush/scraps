use url::Url;

#[derive(Clone)]
pub struct HtmlMetadata {
    title: String,
    description: Option<String>,
    favicon: Option<Url>,
}

impl HtmlMetadata {
    pub fn new(title: &str, description: &Option<String>, favicon: &Option<Url>) -> HtmlMetadata {
        HtmlMetadata {
            title: title.to_string(),
            description: description.clone(),
            favicon: favicon.clone(),
        }
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
