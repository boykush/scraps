use crate::usecase::scrap::lookup_links::usecase::LinkRefKind;
use crate::usecase::scrap::scrap_ref::ScrapRef;
use chrono::SecondsFormat;
use scraps_libs::markdown::query::{CodeBlock, Heading};
use scraps_libs::model::generated::Generated;
use scraps_libs::model::source::Source;
use scraps_libs::model::verification::{TrustTier, Verification};
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

/// How a scrap's content was produced, read from `generated`: `at` is left
/// out when the frontmatter gives none that is readable.
#[derive(Debug, Serialize)]
pub struct GeneratedJson {
    pub by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

impl From<&Generated> for GeneratedJson {
    fn from(generated: &Generated) -> Self {
        Self {
            by: generated.by.clone(),
            at: generated
                .at
                .map(|at| at.to_rfc3339_opts(SecondsFormat::AutoSi, true)),
        }
    }
}

/// A scrap's confirmations, read as an OKF consumer reads `verified`: the
/// trust tier derived from them, and the readable entries newest first.
#[derive(Debug, Serialize)]
pub struct VerifiedJson {
    pub tier: String,
    pub entries: Vec<VerificationJson>,
}

#[derive(Debug, Serialize)]
pub struct VerificationJson {
    pub by: String,
    pub at: String,
}

impl VerifiedJson {
    pub fn new(verifications: &[Verification], tier: TrustTier) -> Self {
        Self {
            tier: tier.as_str().to_string(),
            entries: verifications
                .iter()
                .map(|v| VerificationJson {
                    by: v.by.clone(),
                    at: v.at.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                })
                .collect(),
        }
    }
}

/// One material a scrap derives from, read as an OKF consumer reads an entry
/// of `sources`. Optional keys the entry leaves out are left out here too.
#[derive(Debug, Serialize)]
pub struct SourceJson {
    pub resource: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

impl From<&Source> for SourceJson {
    fn from(source: &Source) -> Self {
        Self {
            resource: source.resource.clone(),
            id: source.id.clone(),
            title: source.title.clone(),
            author: source.author.clone(),
            last_modified: source
                .last_modified
                .map(|at| at.to_rfc3339_opts(SecondsFormat::AutoSi, true)),
        }
    }
}

/// How one scrap references another: in the body, as an embed, or as one of
/// its `sources`.
#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LinkKindJson {
    Link,
    Embed,
    Source,
}

impl From<LinkRefKind> for LinkKindJson {
    fn from(kind: LinkRefKind) -> Self {
        match kind {
            LinkRefKind::Link => LinkKindJson::Link,
            LinkRefKind::Embed => LinkKindJson::Embed,
            LinkRefKind::Source => LinkKindJson::Source,
        }
    }
}
