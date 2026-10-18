use crate::usecase::scrap::scrap_ref::ScrapRef;
use chrono::SecondsFormat;
use scraps_libs::markdown::query::{CodeBlock, Heading};
use scraps_libs::model::verification::{TrustTier, Verification};
use serde::{Deserialize, Serialize};

/// A scrap as an output names it: `key` is what to pass back to a command,
/// `title` is what readers see, and `ctx` is the folder part of the key.
#[derive(Debug, Serialize, Deserialize)]
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

/// A search hit: the scrap, plus its description when it has one so a
/// reader can choose what to open without fetching bodies.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHitJson {
    #[serde(flatten)]
    pub scrap: ScrapRefJson,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

/// A scrap's confirmations, read as an OKF consumer reads `verified`: the
/// trust tier derived from them, and the readable entries newest first.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifiedJson {
    pub tier: String,
    pub entries: Vec<VerificationJson>,
}

#[derive(Debug, Serialize, Deserialize)]
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
