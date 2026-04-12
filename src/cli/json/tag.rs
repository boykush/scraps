use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TagJson {
    pub title: String,
    pub backlinks_count: usize,
}
