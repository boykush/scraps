use comrak::{Arena, parse_document};
use serde_json::Value;
use url::Url;

use super::code_blocks::{CodeBlock, code_blocks_in};
use super::common::options;
use super::frontmatter::frontmatter;
use super::headings::{Heading, headings_in};
use super::images::images_in;
use super::task_items::{TaskItem, task_items_in};
use super::wiki_ref::{WikiRef, wiki_refs};

/// Everything Scraps reads out of one markdown body. Three parses, not one:
/// wiki refs need the embed-exposed text, frontmatter needs the delimiter
/// option, and the remaining queries share a single plain parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapFacts {
    pub refs: Vec<WikiRef>,
    pub headings: Vec<Heading>,
    pub code_blocks: Vec<CodeBlock>,
    pub images: Vec<Url>,
    pub task_items: Vec<TaskItem>,
    pub frontmatter: Option<Value>,
}

impl ScrapFacts {
    pub fn parse(text: &str) -> ScrapFacts {
        let arena = Arena::new();
        let root = parse_document(&arena, text, &options());
        ScrapFacts {
            refs: wiki_refs(text),
            headings: headings_in(root),
            code_blocks: code_blocks_in(root),
            images: images_in(root),
            task_items: task_items_in(root),
            frontmatter: frontmatter(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{code_blocks, headings, images, task_items};
    use super::*;

    const DOC: &str = "# Title\n\nSee [[a]] and #[[t/u]] and ![[b#Sec]].\n\n## Sec\n\n- [ ] open task\n\n```rust\nlet x = 1;\n```\n\n![img](https://example.com/i.png)\n";

    #[test]
    fn it_matches_the_single_queries() {
        let facts = ScrapFacts::parse(DOC);
        assert_eq!(facts.refs, wiki_refs(DOC));
        assert_eq!(facts.headings, headings::headings(DOC));
        assert_eq!(facts.code_blocks, code_blocks::code_blocks(DOC));
        assert_eq!(facts.images, images::images(DOC));
        assert_eq!(facts.task_items, task_items::task_items(DOC));
        assert_eq!(facts.frontmatter, frontmatter(DOC));
    }

    #[test]
    fn it_extracts_every_fact_family() {
        let facts = ScrapFacts::parse(DOC);
        assert_eq!(facts.refs.len(), 3);
        assert_eq!(facts.headings.len(), 2);
        assert_eq!(facts.code_blocks.len(), 1);
        assert_eq!(facts.images.len(), 1);
        assert_eq!(facts.task_items.len(), 1);
        assert!(facts.frontmatter.is_none());
    }

    #[test]
    fn it_reads_frontmatter() {
        let facts = ScrapFacts::parse("---\nstatus: draft\n---\n\nbody\n");
        assert_eq!(
            facts.frontmatter,
            frontmatter("---\nstatus: draft\n---\n\nbody\n")
        );
        assert!(facts.frontmatter.is_some());
    }

    #[test]
    fn it_parses_empty_text() {
        let facts = ScrapFacts::parse("");
        assert!(facts.refs.is_empty());
        assert!(facts.headings.is_empty());
        assert!(facts.frontmatter.is_none());
    }
}
