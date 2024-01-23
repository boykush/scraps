use std::fmt::Display;
use url::form_urlencoded;

use crate::libs::slugify;

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Title {
    v: String,
    pub slug: Slug,
}

impl Title {
    pub fn new(title: &str) -> Title {
        Title {
            v: title.to_owned(),
            slug: Slug::new(title),
        }
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.v)
    }
}

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Slug(String);

impl Slug {
    fn new(v: &str) -> Slug {
        let slug = slugify::by_dash(v);
        Slug(slug)
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url_encoded = form_urlencoded::byte_serialize(self.0.as_bytes()).collect::<String>();
        write!(f, "{}", url_encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let title = Title::new("scrap title");
        assert_eq!(title.v, "scrap title".to_string());
        assert_eq!(title.slug, Slug("scrap-title".to_string()));

        let japanese_title = Title::new("日本語");
        assert_eq!(japanese_title.v, "日本語".to_string());
        assert_eq!(japanese_title.slug, Slug("日本語".to_string()));
        assert_eq!(format!("{}", japanese_title.slug), format!("{}", "%E6%97%A5%E6%9C%AC%E8%AA%9E".to_string()));
    }
}
