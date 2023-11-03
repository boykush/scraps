use super::title::Title;



#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Tag {
    pub title: Title,
}

impl Tag {
    pub fn new(title: &Title) -> Tag {
        Tag {
            title: title.to_owned(),
        }
    }
}

