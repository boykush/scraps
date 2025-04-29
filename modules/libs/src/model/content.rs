use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    pub elements: Vec<ContentElement>,
}

impl Content {
    pub fn new(elements: Vec<ContentElement>) -> Self {
        Self { elements }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentElement {
    Raw(String),
    OGPCard(Url),
}
