use std::collections::HashSet;

use serde_json::Value;
use url::Url;

use crate::markdown::query::{CodeBlock, Heading, ScrapFacts, TaskItem, WikiRef};
use crate::nfc::nfc;

use super::source::{Source, frontmatter_sources};
use super::verification::{TrustTier, Verification, frontmatter_verifications};
use super::{context::Ctx, key::ScrapKey, stem::Stem, tag::Tag};

const FRONTMATTER_TAGS_KEY: &str = "tags";

/// One link to a scrap as it occurs in a source: a `[[wiki-link]]`, or a
/// Markdown link to a `.md` path resolved against the scrap that holds it.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct LinkOccurrence {
    /// `None` when a Markdown link's path climbs out of the wiki root.
    pub target: Option<ScrapKey>,
    pub heading: Option<String>,
    pub line: usize,
    /// The link as written, for a Markdown link; `None` for a wiki-link.
    pub markdown: Option<String>,
}

/// One scrap as the compiler sees it: identity, the source text, and every
/// fact parsed out of that text. `links` is the deduplicated view of
/// `facts.refs`; `tags` is the one of its tag refs and the frontmatter `tags`.
#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    stem: Stem,
    ctx: Option<Ctx>,
    md_text: String,
    facts: ScrapFacts,
    links: Vec<ScrapKey>,
    tags: Vec<Tag>,
    scrap_type: Option<String>,
}

const FRONTMATTER_TITLE_KEY: &str = "title";
const FRONTMATTER_TYPE_KEY: &str = "type";
const FRONTMATTER_DESCRIPTION_KEY: &str = "description";

impl Scrap {
    pub fn self_key(&self) -> ScrapKey {
        ScrapKey::new(&self.stem, &self.ctx)
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
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

    /// Every reference occurrence in source order, duplicates included.
    pub fn refs(&self) -> &[WikiRef] {
        &self.facts.refs
    }

    /// Every link occurrence in source order, duplicates included, each with
    /// the scrap it points at from this scrap's context.
    pub fn link_occurrences(&self) -> impl Iterator<Item = LinkOccurrence> + '_ {
        link_occurrences(&self.facts.refs, &self.ctx)
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

    /// What readers see as the scrap's name: its frontmatter title, or the
    /// file stem when it has none. A title names a scrap and never identifies
    /// it; links resolve by `self_key()`.
    pub fn title(&self) -> &str {
        self.frontmatter_title().unwrap_or(self.stem.as_str())
    }

    /// The frontmatter `title` string, trimmed and otherwise as written. None
    /// when the key is missing, blank or not a string.
    pub fn frontmatter_title(&self) -> Option<&str> {
        self.frontmatter()?
            .get(FRONTMATTER_TITLE_KEY)?
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    /// What kind of thing the scrap is: the frontmatter `type` string,
    /// trimmed and in NFC. A scrap has at most one, and none when the key is
    /// missing, blank or not a string. The value is the author's own
    /// vocabulary.
    pub fn scrap_type(&self) -> Option<&str> {
        self.scrap_type.as_deref()
    }

    /// The author's one-line summary of the scrap: the frontmatter
    /// `description` string, trimmed. None when the key is missing, blank or
    /// not a string.
    pub fn description(&self) -> Option<&str> {
        self.frontmatter()?
            .get(FRONTMATTER_DESCRIPTION_KEY)?
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    /// Who confirmed the scrap's content and when: the readable entries of
    /// the frontmatter `verified`, newest first.
    pub fn verifications(&self) -> Vec<Verification> {
        frontmatter_verifications(self.frontmatter())
    }

    pub fn trust_tier(&self) -> TrustTier {
        TrustTier::of(&self.verifications())
    }

    /// What the scrap's content derives from: the readable entries of the
    /// frontmatter `sources`, in the order written.
    pub fn sources(&self) -> Vec<Source> {
        frontmatter_sources(self.frontmatter())
    }
}

impl Scrap {
    pub fn new(stem: &str, ctx: &Option<Ctx>, text: &str) -> Scrap {
        Scrap::from_facts(stem, ctx, text, ScrapFacts::parse(text))
    }

    /// Assemble a scrap from facts parsed earlier, so a source the on-disk IR
    /// already knows is never parsed twice. The facts are trusted as given.
    pub fn from_facts(stem: &str, ctx: &Option<Ctx>, text: &str, facts: ScrapFacts) -> Scrap {
        // Dedup through a HashSet used to leave the order to chance, which made
        // every consumer of `links()` — backlink lists, rendered pages, the
        // neighborhood walk — reshuffle on each read. Keep first occurrence.
        let mut seen_links = HashSet::new();
        let links: Vec<ScrapKey> = link_occurrences(&facts.refs, ctx)
            .filter_map(|occurrence| occurrence.target)
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

        let scrap_type = frontmatter_type(facts.frontmatter.as_ref());

        Scrap {
            stem: stem.into(),
            ctx: ctx.clone(),
            md_text: text.to_string(),
            facts,
            links,
            tags,
            scrap_type,
        }
    }
}

fn link_occurrences<'a>(
    refs: &'a [WikiRef],
    ctx: &'a Option<Ctx>,
) -> impl Iterator<Item = LinkOccurrence> + 'a {
    refs.iter().filter_map(move |r| match r {
        WikiRef::Link(link) => Some(LinkOccurrence {
            target: Some(ScrapKey::from(link)),
            heading: link.heading.clone(),
            line: link.line,
            markdown: None,
        }),
        WikiRef::MdLink(link) => Some(LinkOccurrence {
            target: link.resolve(ctx),
            heading: link.heading(),
            line: link.line,
            markdown: Some(link.written()),
        }),
        WikiRef::Embed(_) | WikiRef::Tag(_) => None,
    })
}

/// A type is a name like a tag: compared for identity and written as a path.
fn frontmatter_type(frontmatter: Option<&Value>) -> Option<String> {
    frontmatter?
        .get(FRONTMATTER_TYPE_KEY)?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| nfc(value).into_owned())
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
        assert_eq!(scrap.stem(), &"scrap title".into());
        assert!(scrap.ctx().is_none());

        let mut actual_links = scrap.links().to_vec();
        actual_links.sort();

        let mut expected = [
            Stem::from("link1").into(),
            Stem::from("link2").into(),
            ScrapKey::with_ctx(&"link3".into(), &"Context".into()),
        ];
        expected.sort();

        assert_eq!(actual_links, expected);
        assert_eq!(scrap.thumbnail(), None);
    }

    #[test]
    fn it_links_through_markdown_links_to_scrap_files() {
        let scrap = Scrap::new(
            "orders",
            &Some("tables".into()),
            "[customers](/tables/customers.md) [[tables/customers]] \
             [revenue](../metrics/revenue.md#definition) [out](../../out.md) \
             [web](https://example.com/a.md)",
        );

        assert_eq!(
            scrap.links().to_vec(),
            vec![
                ScrapKey::from_path_str("tables/customers"),
                ScrapKey::from_path_str("metrics/revenue"),
            ]
        );

        let occurrences: Vec<LinkOccurrence> = scrap.link_occurrences().collect();
        assert_eq!(occurrences.len(), 4);
        assert_eq!(
            occurrences[0].markdown.as_deref(),
            Some("[customers](/tables/customers.md)")
        );
        assert_eq!(occurrences[1].markdown, None);
        assert_eq!(occurrences[2].heading.as_deref(), Some("definition"));
        assert_eq!(occurrences[3].target, None);
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
                ScrapKey::from(Stem::from("zulu")),
                ScrapKey::from(Stem::from("alpha")),
                ScrapKey::with_ctx(&"mike".into(), &"Context".into()),
            ]
        );
    }

    #[test]
    fn it_new_with_nested_ctx() {
        let scrap = Scrap::new("borrowing", &Some("programming/rust".into()), "body");
        assert_eq!(scrap.stem(), &"borrowing".into());
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
        assert_eq!(scrap.links(), &[ScrapKey::from(Stem::from("bar"))]);
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

    #[rstest::rstest]
    #[case::plain("---\ntype: Blog\n---\n", "Blog")]
    #[case::kept_verbatim("---\ntype: BigQuery Table\n---\n", "BigQuery Table")]
    #[case::case_kept("---\ntype: blog\n---\n", "blog")]
    #[case::trimmed("---\ntype: '  Book  '\n---\n", "Book")]
    #[case::among_other_keys("---\ntitle: T\ntype: Metric\ntags: [a]\n---\n\nbody\n", "Metric")]
    fn it_reads_the_frontmatter_type(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(Scrap::new("foo", &None, text).scrap_type(), Some(expected));
    }

    #[rstest::rstest]
    #[case::no_frontmatter("# Body\n")]
    #[case::no_type_key("---\nstatus: draft\n---\n")]
    #[case::empty("---\ntype: ''\n---\n")]
    #[case::blank("---\ntype: '   '\n---\n")]
    #[case::null("---\ntype:\n---\n")]
    #[case::list("---\ntype: [Blog, Book]\n---\n")]
    #[case::mapping("---\ntype:\n  name: Blog\n---\n")]
    #[case::number("---\ntype: 3\n---\n")]
    #[case::boolean("---\ntype: true\n---\n")]
    #[case::sequence_root("---\n- type\n---\n")]
    fn it_has_no_type_without_a_non_empty_string(#[case] text: &str) {
        assert_eq!(Scrap::new("foo", &None, text).scrap_type(), None);
    }

    #[rstest::rstest]
    #[case::plain(
        "---\ndescription: One row per completed customer order.\n---\n",
        "One row per completed customer order."
    )]
    #[case::trimmed("---\ndescription: '  A summary.  '\n---\n", "A summary.")]
    #[case::folded(
        "---\ndescription: >\n  A folded\n  summary.\n---\n",
        "A folded summary."
    )]
    #[case::among_other_keys(
        "---\ntype: Table\ndescription: Orders.\ntags: [a]\n---\n\nbody\n",
        "Orders."
    )]
    fn it_reads_the_frontmatter_description(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(Scrap::new("foo", &None, text).description(), Some(expected));
    }

    #[rstest::rstest]
    #[case::no_frontmatter("# Body\n")]
    #[case::no_description_key("---\ntype: Blog\n---\n")]
    #[case::empty("---\ndescription: ''\n---\n")]
    #[case::blank("---\ndescription: '   '\n---\n")]
    #[case::null("---\ndescription:\n---\n")]
    #[case::list("---\ndescription: [a, b]\n---\n")]
    #[case::mapping("---\ndescription:\n  text: a\n---\n")]
    #[case::number("---\ndescription: 3\n---\n")]
    #[case::boolean("---\ndescription: true\n---\n")]
    fn it_has_no_description_without_a_non_empty_string(#[case] text: &str) {
        assert_eq!(Scrap::new("foo", &None, text).description(), None);
    }

    #[rstest::rstest]
    #[case::plain("---\ntitle: Customer Orders\n---\n", "Customer Orders")]
    #[case::trimmed("---\ntitle: '  Customer Orders  '\n---\n", "Customer Orders")]
    #[case::symbols(
        "---\ntitle: 'What is C#? [draft] | 50%'\n---\n",
        "What is C#? [draft] | 50%"
    )]
    #[case::among_other_keys("---\ntype: Table\ntitle: Orders\ntags: [a]\n---\n\nbody\n", "Orders")]
    fn it_reads_the_frontmatter_title(#[case] text: &str, #[case] expected: &str) {
        let scrap = Scrap::new("orders", &Some("tables".into()), text);
        assert_eq!(scrap.title(), expected);
        assert_eq!(scrap.frontmatter_title(), Some(expected));
    }

    #[rstest::rstest]
    #[case::no_frontmatter("# Body\n")]
    #[case::no_title_key("---\ntype: Blog\n---\n")]
    #[case::empty("---\ntitle: ''\n---\n")]
    #[case::blank("---\ntitle: '   '\n---\n")]
    #[case::null("---\ntitle:\n---\n")]
    #[case::list("---\ntitle: [a, b]\n---\n")]
    #[case::mapping("---\ntitle:\n  text: a\n---\n")]
    #[case::number("---\ntitle: 3\n---\n")]
    #[case::boolean("---\ntitle: true\n---\n")]
    fn it_is_titled_by_its_stem_without_a_non_empty_string(#[case] text: &str) {
        let scrap = Scrap::new("orders", &Some("tables".into()), text);
        assert_eq!(scrap.title(), "orders");
        assert_eq!(scrap.frontmatter_title(), None);
    }

    #[test]
    fn it_keeps_its_key_whatever_the_title() {
        let scrap = Scrap::new(
            "orders",
            &Some("tables".into()),
            "---\ntitle: Customer Orders\n---\n",
        );
        assert_eq!(scrap.self_key(), ScrapKey::from_path_str("tables/orders"));
        assert_eq!(scrap.stem(), &"orders".into());
    }

    #[test]
    fn it_leaves_frontmatter_and_tags_untouched_by_the_type() {
        let scrap = Scrap::new("foo", &None, "---\ntype: Blog\n---\n\n#[[ai]]\n");
        assert_eq!(
            scrap.frontmatter().and_then(|v| v.get("type")),
            Some(&Value::String("Blog".to_string()))
        );
        assert_eq!(scrap.tags(), &[Tag::from("ai")]);
    }

    // ガイド and タグ, once decomposed (as macOS tools name files) and once
    // composed (as text is typed).
    const GUIDE_NFD: &str = "\u{30ab}\u{3099}\u{30a4}\u{30c8}\u{3099}";
    const GUIDE_NFC: &str = "\u{30ac}\u{30a4}\u{30c9}";
    const TAG_NFD: &str = "\u{30bf}\u{30af}\u{3099}";
    const TAG_NFC: &str = "\u{30bf}\u{30b0}";

    #[test]
    fn it_links_to_one_key_from_either_normalization_form() {
        let text = format!(
            "[[{GUIDE_NFD}]] [[{GUIDE_NFC}]] [a](/{GUIDE_NFD}.md) [b](/{GUIDE_NFC}.md) \
             [c](/%E3%82%AB%E3%82%99%E3%82%A4%E3%83%88%E3%82%99.md)"
        );
        let scrap = Scrap::new("from", &None, &text);
        assert_eq!(scrap.links(), &[ScrapKey::from(Stem::from(GUIDE_NFC))]);
        assert_eq!(
            Scrap::new(GUIDE_NFD, &None, "").self_key(),
            scrap.links()[0]
        );
    }

    #[test]
    fn it_tags_once_from_either_normalization_form() {
        let text = format!("---\ntags: [{TAG_NFD}]\n---\n\n#[[{TAG_NFD}]] #[[{TAG_NFC}]]\n");
        let scrap = Scrap::new("foo", &None, &text);
        assert_eq!(scrap.tags(), &[Tag::from(TAG_NFC)]);
        assert_eq!(scrap.tags()[0].to_string(), TAG_NFC);
    }

    #[test]
    fn it_reads_the_type_in_nfc_and_leaves_the_frontmatter_as_written() {
        let text = format!("---\ntype: {GUIDE_NFD}\nowner: {GUIDE_NFD}\n---\n\n{GUIDE_NFD}\n");
        let scrap = Scrap::new("foo", &None, &text);
        assert_eq!(scrap.scrap_type(), Some(GUIDE_NFC));
        assert_eq!(
            scrap.frontmatter(),
            Some(&json!({"type": GUIDE_NFD, "owner": GUIDE_NFD}))
        );
        assert_eq!(scrap.md_text(), text);
    }

    #[test]
    fn it_reads_the_frontmatter_verified() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "---\nverified:\n  by: human:alice\n  at: 2026-10-17T00:00:00Z\n---\n\nbody\n",
        );
        assert_eq!(scrap.verifications()[0].by, "human:alice");
        assert_eq!(scrap.trust_tier(), TrustTier::HumanReviewed);
    }

    #[test]
    fn it_reads_the_frontmatter_sources() {
        let scrap = Scrap::new(
            "foo",
            &None,
            "---\nsources:\n  - resource: https://example.com\n---\n\nbody\n",
        );
        assert_eq!(scrap.sources()[0].resource, "https://example.com");
    }

    #[test]
    fn it_is_unverified_without_a_frontmatter_verified() {
        let scrap = Scrap::new("foo", &None, "# Body\n");
        assert!(scrap.verifications().is_empty());
        assert_eq!(scrap.trust_tier(), TrustTier::Unverified);
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
