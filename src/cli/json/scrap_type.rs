use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeJson {
    #[serde(rename = "type")]
    pub name: String,
    pub scraps_count: usize,
}
