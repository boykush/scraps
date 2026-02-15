use serde::Deserialize;

use crate::usecase::build::model::sort::SortKey;

#[derive(Deserialize)]
#[serde(remote = "SortKey", rename_all = "snake_case")]
pub enum SerdeSortKey {
    CommittedDate,
    LinkedCount,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SortKeyConfig(#[serde(with = "SerdeSortKey")] SortKey);

impl SortKeyConfig {
    pub fn as_sort_key(&self) -> &SortKey {
        &self.0
    }
}
