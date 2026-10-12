use crate::usecase::scrap::scrap_ref::ScrapRef;
use scraps_libs::markdown::query::{CodeBlock, Heading};
use serde::Serialize;

/// A scrap as a tool result names it: `key` is what to pass back to a tool,
/// `title` is what readers see, and `ctx` is the folder part of the key.
#[derive(Debug, Serialize)]
pub struct ScrapRefJson {
    pub key: String,
    pub title: String,
    pub ctx: Option<String>,
}

impl From<&ScrapRef> for ScrapRefJson {
    fn from(scrap: &ScrapRef) -> Self {
        ScrapRefJson {
            key: scrap.key.to_string(),
            title: scrap.title.clone(),
            ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
        }
    }
}

/// A search hit: the scrap, plus its description when it has one so an
/// agent can choose what to read without fetching bodies.
#[derive(Debug, Serialize)]
pub struct SearchHitJson {
    #[serde(flatten)]
    pub scrap: ScrapRefJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HeadingJson {
    pub level: u8,
    pub text: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

impl From<Heading> for HeadingJson {
    fn from(h: Heading) -> Self {
        Self {
            level: h.level,
            text: h.text,
            line: h.line,
            parent: h.parent,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CodeBlockJson {
    pub lang: Option<String>,
    pub content: String,
    pub line: usize,
}

impl From<CodeBlock> for CodeBlockJson {
    fn from(c: CodeBlock) -> Self {
        Self {
            lang: c.lang,
            content: c.content,
            line: c.line,
        }
    }
}
