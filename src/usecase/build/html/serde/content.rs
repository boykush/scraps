use scraps_libs::model::content::{Content, ContentElement};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
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

#[derive(Clone, PartialEq, Debug)]
pub struct ContentTera(Content);

impl Serialize for ContentTera {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Content", 1)?;
        let elements: Vec<ContentElementTera> = self
            .0
            .elements()
            .iter()
            .map(|element| ContentElementTera(element.clone()))
            .collect();
        state.serialize_field("elements", &elements)?;
        state.end()
    }
}

impl From<Content> for ContentTera {
    fn from(content: Content) -> Self {
        ContentTera(content)
    }
}
