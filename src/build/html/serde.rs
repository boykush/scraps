use crate::build::model::scrap::Scrap;

#[derive(serde::Serialize)]
#[serde(remote = "Scrap")]
struct SerializeScrap {
    title: String,
    links: Vec<String>,
    html_content: String,
}

#[derive(serde::Serialize)]
pub struct SScrap(#[serde(with = "SerializeScrap")] pub Scrap);

#[derive(serde::Serialize)]
pub struct SScraps(pub Vec<SScrap>);
