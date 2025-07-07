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

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_content_display_single_element() {
        let elements = vec![ContentElement::Raw("Hello World".to_string())];
        let content = Content::new(elements);
        assert_eq!(format!("{}", content), "Hello World");
    }

    #[test]
    fn test_content_display_multiple_elements() {
        let elements = vec![
            ContentElement::Raw("Hello ".to_string()),
            ContentElement::Raw("World".to_string()),
        ];
        let content = Content::new(elements);
        assert_eq!(format!("{}", content), "Hello World");
    }

    #[test]
    fn test_content_display_empty() {
        let content = Content::new(vec![]);
        assert_eq!(format!("{}", content), "");
    }

    #[test]
    fn test_content_display_with_autolink() {
        let url = Url::parse("https://example.com").unwrap();
        let elements = vec![
            ContentElement::Raw("Visit ".to_string()),
            ContentElement::Autolink(url),
            ContentElement::Raw(" for more info".to_string()),
        ];
        let content = Content::new(elements);
        assert_eq!(format!("{}", content), "Visit https://example.com/ for more info");
    }

    #[test]
    fn test_content_element_raw_display() {
        let element = ContentElement::Raw("Test text".to_string());
        assert_eq!(format!("{}", element), "Test text");
    }

    #[test]
    fn test_content_element_raw_display_special_chars() {
        let element = ContentElement::Raw("Hello\nWorld\t!".to_string());
        assert_eq!(format!("{}", element), "Hello\nWorld\t!");
    }

    #[test]
    fn test_content_element_raw_display_unicode() {
        let element = ContentElement::Raw("こんにちは世界".to_string());
        assert_eq!(format!("{}", element), "こんにちは世界");
    }

    #[test]
    fn test_content_element_autolink_display() {
        let url = Url::parse("https://example.com").unwrap();
        let element = ContentElement::Autolink(url);
        assert_eq!(format!("{}", element), "https://example.com/");
    }
}
