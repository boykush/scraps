use crate::build::model::scrap::Title;

#[derive(Eq, Hash, PartialEq, Debug)]
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
