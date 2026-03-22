use crate::usecase::build::model::sort::SortKey;

#[derive(serde::Serialize)]
#[serde(remote = "SortKey")]
enum SerializeSortKey {
    #[serde(rename = "committed date")]
    CommittedDate,
    #[serde(rename = "linked count")]
    LinkedCount,
}

#[derive(serde::Serialize, Debug)]
pub struct SortKeyTera(#[serde(with = "SerializeSortKey")] SortKey);

impl From<SortKey> for SortKeyTera {
    fn from(sort_key: SortKey) -> Self {
        SortKeyTera(sort_key)
    }
}
