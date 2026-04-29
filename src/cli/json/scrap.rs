use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapKeyJson {
    pub title: String,
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapJson {
    pub title: String,
    pub ctx: Option<String>,
    pub md_text: String,
}
