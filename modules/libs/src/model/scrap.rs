use std::collections::HashSet;

use serde_json::Value;
use url::Url;

use crate::markdown::query::{CodeBlock, Heading, ScrapFacts, TaskItem, WikiRef};

use super::{context::Ctx, key::ScrapKey, tag::Tag, title::Title};

const FRONTMATTER_TAGS_KEY: &str = "tags";

/// One scrap as the compiler sees it: identity, the source text, and every
/// fact parsed out of that text. `links` is the deduplicated view of
/// `facts.refs`; `tags` is the one of its tag refs and the frontmatter `tags`.
#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    title: Title,
    ctx: Option<Ctx>,
    md_text: String,
    facts: ScrapFacts,
    links: Vec<ScrapKey>,
    tags: Vec<Tag>,
}

impl Scrap {
    pub fn self_key(&self) -> ScrapKey {
        ScrapKey::new(&self.title, &self.ctx)
    }

    pub fn title(&self) -> &Title {
        &self.title
    }

    pub fn ctx(&self) -> &Option<Ctx> {
        &self.ctx
    }

    pub fn links(&self) -> &[ScrapKey] {
        &self.links
    }

    /// The scrap's tags: `#[[tag]]` occurrences in the body, then the strings
    /// of the frontmatter `tags` list, with duplicates removed (first
    /// occurrence kept).
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    pub fn md_text(&self) -> &str {
        &self.md_text
    }

    pub fn thumbnail(&self) -> Option<Url> {
        self.facts.images.first().cloned()
    }

    pub fn facts(&self) -> &ScrapFacts {
        &self.facts
    }

    /// Every `[[]]`-family occurrence in source order, duplicates included.
    pub fn refs(&self) -> &[WikiRef] {
        &self.facts.refs
    }

    pub fn headings(&self) -> &[Heading] {
        &self.facts.headings
    }

    pub fn code_blocks(&self) -> &[CodeBlock] {
        &self.facts.code_blocks
    }

    pub fn images(&self) -> &[Url] {
        &self.facts.images
    }

    pub fn task_items(&self) -> &[TaskItem] {
        &self.facts.task_items
    }

    pub fn frontmatter(&self) -> Option<&Value> {
        self.facts.frontmatter.as_ref()
    }
}

impl Scrap {
    pub fn new(title: &str, ctx: &Option<Ctx>, text: &str) -> Scrap {
        Scrap::from_facts(title, ctx, text, ScrapFacts::parse(text))
    }

    /// Assemble a scrap from facts parsed earlier, so a source the on-disk IR
    /// already knows is never parsed twice. The facts are trusted as given.
    pub fn from_facts(title: &str, ctx: &Option<Ctx>, text: &str, facts: ScrapFacts) -> Scrap {
        // Dedup through a HashSet used to leave the order to chance, which made
        // every consumer of `links()` — backlink lists, rendered pages, the
        // neighborhood walk — reshuffle on each read. Keep first occurrence.
        let mut seen_links = HashSet::new();
        let links: Vec<ScrapKey> = facts
            .refs
            .iter()
            .filter_map(|r| match r {
                WikiRef::Link(link) => Some(ScrapKey::from(link)),
                _ => None,
            })
            .filter(|key| seen_links.insert(key.clone()))
            .collect();

        // Tags dedup within this scrap in first-occurrence order, body before
        // frontmatter; cross-scrap aggregation is `Tags::new`'s job.
        let mut seen_tags = HashSet::new();
        let tags: Vec<Tag> = facts
            .refs
            .iter()
            .filter_map(|r| match r {
                WikiRef::Tag(tag) => Some(Tag::from(tag.path.join("/").as_str())),
                _ => None,
            })
            .chain(frontmatter_tags(facts.frontmatter.as_ref()))
            .filter(|tag| seen_tags.insert(tag.clone()))
            .collect();

        Scrap {
            title: title.into(),
            ctx: ctx.clone(),
            md_text: text.to_string(),
            facts,
            links,
            tags,
        }
    }
}

/// The strings of the frontmatter `tags` list, read as tag paths. Frontmatter
/// is the author's own data, so anything else there (no such key, a non-list
/// value, a non-string entry) is skipped rather than reported. An entry with
/// an empty path segment is no tag, as `#[[a/]]` is none in the body.
fn frontmatter_tags(frontmatter: Option<&Value>) -> impl Iterator<Item = Tag> + '_ {
    frontmatter
        .and_then(|fm| fm.get(FRONTMATTER_TAGS_KEY))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|path| path.split('/').all(|segment| !segment.is_empty()))
        .map(Tag::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    // v1 shape: `Scrap::new` takes `ctx: &Option<Ctx>`. None = root scrap,
    // Some(ctx) carries a multi-segment context.
    #[test]
    fn it_new_at_root() {
        let scrap = Scrap::new("scrap title", &None, "[[link1]][[link2]][[Context/link3]]");
        assert_eq!(scrap.title(), &"scrap title".into());
        assert!(scrap.ctx().is_none());

        let mut actual_links = scrap.links().to_vec();
        actual_links.sort();

        let mut expected = [
            Title::from("link1").into(),
            Title::from("link2").into(),
            ScrapKey::with_ctx(&"link3".into(), &"Context".into()),
        ];
        expected.sort();

        assert_eq!(actual_links, expected);
        assert_eq!(scrap.thumbnail(), None);
    }

    #[test]
    fn it_keeps_links_in_first_occurrence_order() {
        let scrap = Scrap::new(
            "scrap title",
            &None,
            "[[zulu]] [[alpha]] [[zulu]] [[Context/mike]]",
        );

        assert_eq!(
            scrap.links().to_vec(),
            vec![
                ScrapKey::from(Title::from("zulu")),
                ScrapKey::from(Title::from("alpha")),
                ScrapKey::with_ctx(&"mike".into(), &"Context".into()),
            ]
        );
    }

    #[test]
    fn it_new_with_nested_ctx() {
        let scrap = Scrap::new("borrowing", &Some("programming/rust".into()), "body");
        assert_eq!(scrap.title(), &"borrowing".into());
        let ctx = scrap.ctx().as_ref().expect("ctx should be Some");
        assert_eq!(format!("{}", ctx), "programming/rust");
        assert_eq!(ctx.depth(), 2);
    }

    #[test]
    fn it_self_key_includes_ctx() {
        let scrap = Scrap::new("foo", &Some("a/b".into()), "");
        let key = scrap.self_key();
        assert_eq!(format!("{}", key), "a/b/foo");
    }

    #[test]
    fn it_self_key_at_root() {
        let scrap = Scrap::new("foo", &None, "");
        let key = scrap.self_key();
        assert_eq!(format!("{}", key), "foo");
    }

    // v1 shape: Scrap exposes explicitly-declared `#[[tag]]` occurrences via
    // `tags()`, populated at construction time from the markdown body.
    // Implicit derivation from unresolved `[[]]` links is being removed.
    #[test]
    fn it_tags_extracted_from_body() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "body with #[[ai]] and #[[programming/rust]] tags",
        );
        let mut got: Vec<String> = scrap.tags().iter().map(|t| format!("{}", t)).collect();
        got.sort();
        assert_eq!(got, vec!["ai".to_string(), "programming/rust".to_string()]);
    }

    #[test]
    fn it_tags_empty_when_no_explicit_tags() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "body with [[wikilink]] but no hashtag tags here",
        );
        assert!(scrap.tags().is_empty());
    }

    #[test]
    fn it_tags_dedup_within_scrap() {
        let scrap = Scrap::new("foo", &None, "#[[ai]] then #[[ai]] again");
        assert_eq!(scrap.tags().len(), 1);
    }

    #[test]
    fn it_tags_excluded_from_code_blocks() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "real #[[ai]]\n```\n#[[code-only]]\n```\n`#[[inline-code]]`",
        );
        let names: Vec<String> = scrap.tags().iter().map(|t| format!("{}", t)).collect();
        assert_eq!(names, vec!["ai".to_string()]);
    }

    #[test]
    fn it_tags_union_body_and_frontmatter() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "---\ntags: [sales, a/b]\n---\n\n#[[body]] and #[[sales]]\n",
        );
        assert_eq!(
            scrap.tags(),
            &[Tag::from("body"), Tag::from("sales"), Tag::from("a/b")]
        );
    }

    #[rstest]
    #[case::no_tags_key("---\nstatus: draft\n---\n")]
    #[case::scalar("---\ntags: sales\n---\n")]
    #[case::mapping("---\ntags:\n  sales: true\n---\n")]
    #[case::null("---\ntags:\n---\n")]
    #[case::sequence_root("---\n- tags\n---\n")]
    #[case::non_string_entries("---\ntags: [1, true, null, [nested], {k: v}]\n---\n")]
    #[case::empty_segments("---\ntags: ['', '/', 'a/', '/a', 'a//b']\n---\n")]
    fn it_ignores_frontmatter_tags_it_cannot_read(#[case] text: &str) {
        assert!(Scrap::new("foo", &None, text).tags().is_empty());
    }

    #[test]
    fn it_keeps_the_readable_frontmatter_tags_among_unreadable_ones() {
        let scrap = Scrap::new("foo", &None, "---\ntags: [1, sales, 'a/', sales]\n---\n");
        assert_eq!(scrap.tags(), &[Tag::from("sales")]);
    }

    #[test]
    fn it_leaves_frontmatter_and_refs_untouched_by_frontmatter_tags() {
        let scrap = Scrap::new("foo", &None, "---\ntags: [sales]\n---\n\nbody\n");
        assert_eq!(scrap.frontmatter(), Some(&json!({"tags": ["sales"]})));
        assert!(scrap.refs().is_empty());
    }

    #[test]
    fn it_keeps_every_parsed_fact() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "# Foo\n\n## Notes\n\n- [ ] todo\n\n```rust\nfn x() {}\n```\n\n![img](https://example.com/i.png) [[bar#Notes|Bar]] #[[ai]] ![[baz]]",
        );
        assert_eq!(scrap.headings().len(), 2);
        assert_eq!(scrap.headings()[1].text, "Notes");
        assert_eq!(scrap.code_blocks().len(), 1);
        assert_eq!(scrap.code_blocks()[0].lang.as_deref(), Some("rust"));
        assert_eq!(scrap.task_items().len(), 1);
        assert_eq!(scrap.images().len(), 1);
        assert_eq!(scrap.thumbnail(), scrap.images().first().cloned());
        assert_eq!(scrap.refs().len(), 3);
        assert_eq!(scrap.links(), &[ScrapKey::from(Title::from("bar"))]);
        assert_eq!(scrap.tags(), &[Tag::from("ai")]);
        assert!(scrap.frontmatter().is_none());
    }

    #[test]
    fn it_reads_frontmatter() {
        let scrap = Scrap::new("foo", &None, "---\nstatus: draft\n---\n\nbody\n");
        assert_eq!(
            scrap
                .frontmatter()
                .and_then(|v| v.get("status"))
                .and_then(|v| v.as_str()),
            Some("draft")
        );
    }

    #[test]
    fn it_from_facts_equals_new() {
        let text = "# T\n\n[[a]] [[a]] #[[b]] ![[c]]\n\n- [x] done\n";
        let facts = ScrapFacts::parse(text);
        assert_eq!(
            Scrap::from_facts("t", &Some("ctx".into()), text, facts),
            Scrap::new("t", &Some("ctx".into()), text)
        );
    }

    #[test]
    fn it_from_facts_trusts_the_given_facts() {
        let facts = ScrapFacts {
            headings: vec![Heading {
                level: 1,
                text: "Ghost".to_string(),
                line: 1,
                parent: None,
            }],
            ..ScrapFacts::parse("")
        };
        let scrap = Scrap::from_facts("t", &None, "plain text without headings", facts);
        assert_eq!(scrap.headings()[0].text, "Ghost");
    }
}
