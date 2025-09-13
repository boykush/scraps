use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ScrapJson {
    pub title: String,
    pub ctx: Option<String>,
    pub md_text: String,
}
