use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapKeyJson {
    pub title: String,
    pub ctx: Option<String>,
}
