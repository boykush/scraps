use std::fmt;

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
    Shortcode(String),
}

impl fmt::Display for ContentElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentElement::Raw(text) => write!(f, "{}", text),
            ContentElement::Shortcode(code) => write!(f, "{}", code),
        }
    }
}
