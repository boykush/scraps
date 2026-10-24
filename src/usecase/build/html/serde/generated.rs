use scraps_libs::model::generated::Generated;

/// Who wrote a scrap's content, as its `generated` names it. The date is left
/// to the committed date, so the page names the actor only.
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
