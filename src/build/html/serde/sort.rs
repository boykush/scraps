use crate::build::model::sort::SortKey;

#[derive(serde::Serialize)]
pub enum SerializeSortKey {
    #[serde(rename = "committed date")]
    CommittedDate,
    #[serde(rename = "linked count")]
    LinkedCount,
}

impl From<SortKey> for SerializeSortKey {
    fn from(sort_key: SortKey) -> Self {
        match sort_key {
            SortKey::CommittedDate => SerializeSortKey::CommittedDate,
            SortKey::LinkedCount => SerializeSortKey::LinkedCount,
        }
    }
}
