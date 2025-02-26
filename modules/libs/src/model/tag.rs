use super::title::Title;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Tag {
    pub title: Title,
}

impl From<Title> for Tag {
    fn from(title: Title) -> Self {
        Tag { title }
    }
}

impl From<&str> for Tag {
    fn from(title: &str) -> Self {
        Tag {
            title: title.into(),
        }
    }
}
