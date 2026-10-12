use comrak::{Arena, parse_document};
use serde_json::Value;
use url::Url;

use super::code_blocks::{CodeBlock, code_blocks_in};
use super::common::options;
use super::frontmatter::read;
use super::headings::{Heading, headings_in};
use super::images::images_in;
use super::task_items::{TaskItem, task_items_in};
use super::wiki_ref::{WikiRef, wiki_refs};

/// Everything Scraps reads out of one scrap. Three parses, not one:
/// frontmatter needs the delimiter option, wiki refs need the embed-exposed
/// text, and the remaining queries share a single plain parse. All but the
/// first read the text with the frontmatter block masked.
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
        let (frontmatter, text) = read(text);
        let root = parse_document(&arena, &text, &options());
        ScrapFacts {
            refs: wiki_refs(&text),
            headings: headings_in(root),
            code_blocks: code_blocks_in(root),
            images: images_in(root),
            task_items: task_items_in(root),
            frontmatter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::frontmatter::frontmatter;
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

    const FRONTED: &str = "---\ntitle: \"see [[hidden]] and #[[secret]]\"\ntodo:\n  - \"[ ] not a task\"\nimage: \"![i](https://example.com/y.png)\"\n---\n\n# Schema\n\nSee [[a]].\n\n- [ ] real task\n\n```rust\nlet x = 1;\n```\n";

    #[test]
    fn it_reads_no_fact_out_of_the_frontmatter_block() {
        let facts = ScrapFacts::parse(FRONTED);
        let headings: Vec<_> = facts.headings.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(headings, vec!["Schema"]);
        assert_eq!(facts.refs.len(), 1);
        assert!(matches!(&facts.refs[0], WikiRef::Link(l) if l.title == "a"));
        assert_eq!(facts.task_items.len(), 1);
        assert_eq!(facts.task_items[0].text, "real task");
        assert_eq!(facts.code_blocks.len(), 1);
        assert!(facts.images.is_empty());
        assert!(facts.frontmatter.is_some());
    }

    #[test]
    fn it_keeps_the_line_numbers_of_the_file() {
        let facts = ScrapFacts::parse(FRONTED);
        assert_eq!(facts.headings[0].line, 8);
        assert!(matches!(&facts.refs[0], WikiRef::Link(l) if l.line == 10));
        assert_eq!(facts.task_items[0].line, 12);
        assert_eq!(facts.code_blocks[0].line, 14);
    }

    #[test]
    fn it_matches_the_single_queries_under_frontmatter() {
        let facts = ScrapFacts::parse(FRONTED);
        assert_eq!(facts.refs, wiki_refs(FRONTED));
        assert_eq!(facts.headings, headings::headings(FRONTED));
        assert_eq!(facts.code_blocks, code_blocks::code_blocks(FRONTED));
        assert_eq!(facts.images, images::images(FRONTED));
        assert_eq!(facts.task_items, task_items::task_items(FRONTED));
    }

    #[test]
    fn it_reads_an_unrecognised_block_as_content() {
        let facts = ScrapFacts::parse("---\nsee: [[kept]\n---\n\n# Schema\n");
        assert_eq!(facts.frontmatter, None);
        assert_eq!(facts.headings.len(), 2);
    }
}
