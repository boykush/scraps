use std::collections::HashSet;

use url::Url;

use crate::markdown;

use super::{context::Ctx, key::ScrapKey, tag::Tag, title::Title};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    title: Title,
    ctx: Option<Ctx>,
    links: Vec<ScrapKey>,
    tags: Vec<Tag>,
    md_text: String,
    thumbnail: Option<Url>,
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

    /// Explicitly-declared `#[[tag]]` tags found in the body, in occurrence
    /// order with duplicates removed (first occurrence kept).
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    pub fn md_text(&self) -> &str {
        &self.md_text
    }

    pub fn thumbnail(&self) -> Option<Url> {
        self.thumbnail.clone()
    }
}

impl Scrap {
    pub fn new(title: &str, ctx: &Option<Ctx>, text: &str) -> Scrap {
        let links: Vec<ScrapKey> = markdown::query::wikilinks(text)
            .iter()
            .map(ScrapKey::from)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let thumbnail = markdown::query::images(text).into_iter().next();

        // Build the tag list from explicit `#[[tag]]` occurrences. Preserve
        // the first occurrence order and drop duplicates within this scrap;
        // cross-scrap aggregation is `Tags::new`'s job.
        let mut seen = HashSet::new();
        let tags: Vec<Tag> = markdown::query::tags(text)
            .into_iter()
            .map(|occ| Tag::from(occ.path.join("/").as_str()))
            .filter(|tag| seen.insert(tag.clone()))
            .collect();

        Scrap {
            title: title.into(),
            ctx: ctx.clone(),
            links,
            tags,
            md_text: text.to_string(),
            thumbnail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
