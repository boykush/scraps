use scraps_libs::model::content::{Content, ContentElement};
use serde::Serialize;
use url::Url;

fn serialize_content_element<S>(element: &ContentElement, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match element {
        ContentElement::Raw(text) => {
            serializer.serialize_newtype_variant("ContentElement", 0, "raw", text)
        }
        ContentElement::Autolink(url) => serializer.serialize_newtype_variant(
            "ContentElement",
            1,
            "autolink",
            &ContentElementAutolink {
                url: url.clone(),
                host: url.host_str().map(|s| s.to_string()),
            },
        ),
    }
}

#[derive(Serialize)]
struct ContentElementAutolink {
    url: Url,
    host: Option<String>,
}

#[derive(Serialize)]
struct ContentElementTera(#[serde(serialize_with = "serialize_content_element")] ContentElement);

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
