use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TagJson {
    pub title: String,
    pub backlinks_count: usize,
}
