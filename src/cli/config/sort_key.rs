use serde::Deserialize;

use crate::build::model::sort::SortKey;

#[derive(Deserialize)]
#[serde(remote = "SortKey", rename_all = "snake_case")]
pub enum SerdeSortKey {
    CommittedDate,
    LinkedCount,
}

#[derive(Deserialize, Debug)]
pub struct SortKeyConfig(#[serde(with = "SerdeSortKey")] SortKey);

impl SortKeyConfig {
    pub fn into_sort_key(self) -> SortKey {
        self.0.clone()
    }
}