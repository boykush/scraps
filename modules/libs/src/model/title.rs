use std::fmt::Display;

use super::slug::Slug;

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Title {
    v: String,
    pub slug: Slug,
}

impl Title {
    pub fn new(title: &str) -> Title {
        Title {
            v: title.to_string(),
            slug: Slug::new(title),
        }
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.v)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_new() {
        let title = Title::new("scrap title");
        assert_eq!(title.v, "scrap title".to_string());
        assert_eq!(title.slug, Slug::new("scrap-title"));
    }
}
