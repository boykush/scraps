use scraps_libs::markdown::query::{CodeBlock, Heading};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapKeyJson {
    pub title: String,
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapJson {
    pub title: String,
    pub ctx: Option<String>,
    pub md_text: String,
    pub headings: Vec<HeadingJson>,
    pub code_blocks: Vec<CodeBlockJson>,
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
