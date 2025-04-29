use scraps_libs::model::content::{Content, ContentElement};
use serde::Serialize;
use url::Url;

#[derive(Serialize)]
#[serde(remote = "ContentElement")]
enum SerializeContentElement {
    #[serde(rename = "raw")]
    Raw(String),
    #[serde(rename = "ogp_card")]
    OGPCard(Url),
}

#[derive(Serialize)]
struct ContentElementTera(#[serde(with = "SerializeContentElement")] ContentElement);

#[derive(Serialize)]
#[serde(remote = "Content")]
pub struct SerializeContent {
    #[serde(serialize_with = "serialize_content_elements")]
    elements: Vec<ContentElement>,
}

fn serialize_content_elements<S>(
    elements: &[ContentElement],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let serialized_elements: Vec<ContentElementTera> = elements
        .iter()
        .map(|element| ContentElementTera(element.clone()))
        .collect();
    serialized_elements.serialize(serializer)
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct ContentTera(#[serde(with = "SerializeContent")] Content);

impl From<Content> for ContentTera {
    fn from(content: Content) -> Self {
        ContentTera(content)
    }
}
