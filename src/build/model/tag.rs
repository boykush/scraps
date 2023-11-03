use super::scrap::{Slug, Title};

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Tag {
    pub title: Title,
    pub slug: Slug
}

impl Tag {
    pub fn new(title: &Title) -> Tag {
        let slug = title.to_owned().to_slug();
        Tag {
            title: title.to_owned(),
            slug: slug
        }
    }
}
