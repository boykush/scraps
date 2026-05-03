use std::collections::HashSet;

use url::Url;

use crate::markdown;

use super::{context::Ctx, key::ScrapKey, title::Title};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    title: Title,
    ctx: Option<Ctx>,
    links: Vec<ScrapKey>,
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

        Scrap {
            title: title.into(),
            ctx: ctx.clone(),
            links,
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
}
