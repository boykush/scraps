use scraps_libs::model::generated::Generated;

/// Who wrote a scrap's content, as its `generated` names it. The date has a
/// line of its own, filled in from git when `at` is missing.
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct GeneratedTera {
    by: String,
}

impl From<&Generated> for GeneratedTera {
    fn from(generated: &Generated) -> Self {
        GeneratedTera {
            by: generated.by.clone(),
        }
    }
}
