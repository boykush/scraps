use std::fmt;

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

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for element in &self.elements {
            write!(f, "{}", element)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentElement {
    Raw(String),
    Autolink(Url),
}

impl fmt::Display for ContentElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentElement::Raw(text) => write!(f, "{}", text),
            ContentElement::Autolink(url) => write!(f, "{}", url),
        }
    }
}
