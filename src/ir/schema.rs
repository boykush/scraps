use scraps_libs::markdown::query::{
    CodeBlock, EmbedRef, Heading, ScrapFacts, TagRef, TaskItem, TaskStatus, WikiLinkRef, WikiRef,
};
use scraps_libs::model::{key::ScrapKey, scrap::Scrap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// Bump whenever the shape or meaning of anything below changes. A file
/// carrying another version is discarded and rebuilt, never migrated.
pub const FORMAT_VERSION: u32 = 1;

/// The whole compiled wiki: one object per source file plus the link table
/// that resolving every `[[ ]]` reference against those objects produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrFile {
    pub format_version: u32,
    pub scraps_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_head: Option<String>,
    pub scraps: Vec<ScrapObject>,
    pub links: Vec<LinkEdge>,
}

impl IrFile {
    pub fn new(scraps: Vec<ScrapObject>, links: Vec<LinkEdge>) -> IrFile {
        IrFile {
            format_version: FORMAT_VERSION,
            scraps_version: env!("CARGO_PKG_VERSION").to_string(),
            git_head: None,
            scraps,
            links,
        }
    }

    pub fn is_current(&self) -> bool {
        self.format_version == FORMAT_VERSION && self.scraps_version == env!("CARGO_PKG_VERSION")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyJson {
    pub title: String,
    pub ctx: Option<String>,
}

impl From<&ScrapKey> for KeyJson {
    fn from(key: &ScrapKey) -> Self {
        KeyJson {
            title: key.title().to_string(),
            ctx: key.ctx().as_ref().map(|c| c.to_string()),
        }
    }
}

/// One source file as the compiler last saw it. `hash` is the content hash
/// the loader compares before trusting the rest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrapObject {
    pub path: String,
    pub hash: String,
    pub title: String,
    pub ctx: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commited_ts: Option<i64>,
    pub refs: Vec<RefJson>,
    pub headings: Vec<HeadingJson>,
    pub code_blocks: Vec<CodeBlockJson>,
    pub images: Vec<String>,
    pub task_items: Vec<TaskItemJson>,
    pub frontmatter: Option<Value>,
}

impl ScrapObject {
    pub fn from_scrap(path: &str, hash: &str, scrap: &Scrap) -> ScrapObject {
        ScrapObject {
            path: path.to_string(),
            hash: hash.to_string(),
            title: scrap.title().to_string(),
            ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
            commited_ts: None,
            refs: scrap.refs().iter().map(RefJson::from).collect(),
            headings: scrap.headings().iter().map(HeadingJson::from).collect(),
            code_blocks: scrap
                .code_blocks()
                .iter()
                .map(CodeBlockJson::from)
                .collect(),
            images: scrap.images().iter().map(|u| u.to_string()).collect(),
            task_items: scrap.task_items().iter().map(TaskItemJson::from).collect(),
            frontmatter: scrap.frontmatter().cloned(),
        }
    }

    pub fn to_facts(&self) -> ScrapFacts {
        ScrapFacts {
            refs: self.refs.iter().cloned().map(WikiRef::from).collect(),
            headings: self.headings.iter().cloned().map(Heading::from).collect(),
            code_blocks: self
                .code_blocks
                .iter()
                .cloned()
                .map(CodeBlock::from)
                .collect(),
            images: self
                .images
                .iter()
                .filter_map(|u| Url::parse(u).ok())
                .collect(),
            task_items: self
                .task_items
                .iter()
                .cloned()
                .map(TaskItem::from)
                .collect(),
            frontmatter: self.frontmatter.clone(),
        }
    }
}

/// A `[[ ]]`-family occurrence. `ctx` is the `/`-joined context path so a
/// reference reads like the `[[Ctx/Title]]` the author wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RefJson {
    Link {
        title: String,
        ctx: Option<String>,
        heading: Option<String>,
        alias: Option<String>,
        line: usize,
    },
    Embed {
        title: String,
        ctx: Option<String>,
        heading: Option<String>,
        line: usize,
    },
    Tag {
        tag: String,
        line: usize,
    },
}

fn join_ctx(ctx_path: &[String]) -> Option<String> {
    if ctx_path.is_empty() {
        None
    } else {
        Some(ctx_path.join("/"))
    }
}

fn split_ctx(ctx: Option<String>) -> Vec<String> {
    ctx.map(|s| s.split('/').map(String::from).collect())
        .unwrap_or_default()
}

impl From<&WikiRef> for RefJson {
    fn from(r: &WikiRef) -> Self {
        match r {
            WikiRef::Link(l) => RefJson::Link {
                title: l.title.clone(),
                ctx: join_ctx(&l.ctx_path),
                heading: l.heading.clone(),
                alias: l.alias.clone(),
                line: l.line,
            },
            WikiRef::Embed(e) => RefJson::Embed {
                title: e.title.clone(),
                ctx: join_ctx(&e.ctx_path),
                heading: e.heading.clone(),
                line: e.line,
            },
            WikiRef::Tag(t) => RefJson::Tag {
                tag: t.path.join("/"),
                line: t.line,
            },
        }
    }
}

impl From<RefJson> for WikiRef {
    fn from(r: RefJson) -> Self {
        match r {
            RefJson::Link {
                title,
                ctx,
                heading,
                alias,
                line,
            } => WikiRef::Link(WikiLinkRef {
                ctx_path: split_ctx(ctx),
                title,
                heading,
                alias,
                line,
            }),
            RefJson::Embed {
                title,
                ctx,
                heading,
                line,
            } => WikiRef::Embed(EmbedRef {
                ctx_path: split_ctx(ctx),
                title,
                heading,
                line,
            }),
            RefJson::Tag { tag, line } => WikiRef::Tag(TagRef {
                path: tag.split('/').map(String::from).collect(),
                line,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingJson {
    pub level: u8,
    pub text: String,
    pub line: usize,
    pub parent: Option<String>,
}

impl From<&Heading> for HeadingJson {
    fn from(h: &Heading) -> Self {
        HeadingJson {
            level: h.level,
            text: h.text.clone(),
            line: h.line,
            parent: h.parent.clone(),
        }
    }
}

impl From<HeadingJson> for Heading {
    fn from(h: HeadingJson) -> Self {
        Heading {
            level: h.level,
            text: h.text,
            line: h.line,
            parent: h.parent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeBlockJson {
    pub lang: Option<String>,
    pub content: String,
    pub line: usize,
}

impl From<&CodeBlock> for CodeBlockJson {
    fn from(c: &CodeBlock) -> Self {
        CodeBlockJson {
            lang: c.lang.clone(),
            content: c.content.clone(),
            line: c.line,
        }
    }
}

impl From<CodeBlockJson> for CodeBlock {
    fn from(c: CodeBlockJson) -> Self {
        CodeBlock {
            lang: c.lang,
            content: c.content,
            line: c.line,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatusJson {
    Open,
    Done,
    Deferred,
}

impl From<&TaskStatus> for TaskStatusJson {
    fn from(s: &TaskStatus) -> Self {
        match s {
            TaskStatus::Open => TaskStatusJson::Open,
            TaskStatus::Done => TaskStatusJson::Done,
            TaskStatus::Deferred => TaskStatusJson::Deferred,
        }
    }
}

impl From<TaskStatusJson> for TaskStatus {
    fn from(s: TaskStatusJson) -> Self {
        match s {
            TaskStatusJson::Open => TaskStatus::Open,
            TaskStatusJson::Done => TaskStatus::Done,
            TaskStatusJson::Deferred => TaskStatus::Deferred,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskItemJson {
    pub status: TaskStatusJson,
    pub text: String,
    pub line: usize,
}

impl From<&TaskItem> for TaskItemJson {
    fn from(t: &TaskItem) -> Self {
        TaskItemJson {
            status: TaskStatusJson::from(&t.status),
            text: t.text.clone(),
            line: t.line,
        }
    }
}

impl From<TaskItemJson> for TaskItem {
    fn from(t: TaskItemJson) -> Self {
        TaskItem {
            status: TaskStatus::from(t.status),
            text: t.text,
            line: t.line,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Link,
    Embed,
}

/// One resolved reference between scraps. `resolved` is false when `to`
/// names no scrap, so consumers see broken structure in place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkEdge {
    pub from: KeyJson,
    pub to: KeyJson,
    pub kind: EdgeKind,
    pub heading: Option<String>,
    pub line: usize,
    pub resolved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::model::{context::Ctx, title::Title};

    const DOC: &str = "---\nstatus: draft\n---\n\n# Title\n\nSee [[a]], [[Ctx/b#Sec|B]], #[[t/u]] and ![[c#Part]].\n\n- [x] done\n\n```rust\nlet x = 1;\n```\n\n![img](https://example.com/i.png)\n";

    #[test]
    fn it_round_trips_facts_through_the_object() {
        let scrap = Scrap::new("t", &Some("a/b".into()), DOC);
        let object = ScrapObject::from_scrap("a/b/t.md", "hash", &scrap);
        assert_eq!(object.to_facts(), *scrap.facts());
        assert_eq!(object.title, "t");
        assert_eq!(object.ctx.as_deref(), Some("a/b"));
    }

    #[test]
    fn it_round_trips_the_object_through_json() {
        let scrap = Scrap::new("t", &None, DOC);
        let object = ScrapObject::from_scrap("t.md", "hash", &scrap);
        let json = serde_json::to_string(&object).unwrap();
        let back: ScrapObject = serde_json::from_str(&json).unwrap();
        assert_eq!(back, object);
    }

    #[test]
    fn it_tags_refs_by_kind() {
        let scrap = Scrap::new("t", &None, "[[a]] #[[t]] ![[c]]");
        let object = ScrapObject::from_scrap("t.md", "hash", &scrap);
        let json = serde_json::to_string(&object.refs).unwrap();
        assert!(json.contains(r#"{"kind":"link","title":"a","ctx":null"#));
        assert!(json.contains(r#"{"kind":"tag","tag":"t","line":1}"#));
        assert!(json.contains(r#"{"kind":"embed","title":"c""#));
    }

    #[test]
    fn it_flattens_a_key_with_ctx() {
        let key = ScrapKey::new(&Title::from("t"), &Some(Ctx::from("a/b")));
        let json = KeyJson::from(&key);
        assert_eq!(json.title, "t");
        assert_eq!(json.ctx.as_deref(), Some("a/b"));
    }

    #[test]
    fn it_reads_an_older_file_shape_without_optional_fields() {
        let json = r#"{"format_version":1,"scraps_version":"x","scraps":[],"links":[]}"#;
        let file: IrFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.git_head, None);
        assert!(!file.is_current());
    }
}
