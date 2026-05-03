use std::fmt;

use super::{context::Ctx, title::Title};

/// A scrap is uniquely identified by a title and an optional hierarchical
/// context. `ctx == None` means the scrap is at the root of the scraps
/// directory; `ctx == Some(_)` carries one or more context segments.
#[derive(PartialEq, Clone, Debug, PartialOrd, Eq, Ord, Hash)]
pub struct ScrapKey {
    title: Title,
    ctx: Option<Ctx>,
}

impl From<Title> for ScrapKey {
    fn from(title: Title) -> Self {
        ScrapKey { title, ctx: None }
    }
}

impl fmt::Display for ScrapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ctx {
            Some(ctx) => write!(f, "{}/{}", ctx, self.title),
            None => write!(f, "{}", self.title),
        }
    }
}

impl From<ScrapKey> for Title {
    fn from(val: ScrapKey) -> Self {
        val.title
    }
}

impl From<&ScrapKey> for Title {
    fn from(val: &ScrapKey) -> Self {
        val.title.clone()
    }
}

impl From<ScrapKey> for Option<Ctx> {
    fn from(val: ScrapKey) -> Self {
        val.ctx
    }
}

impl From<&ScrapKey> for Option<Ctx> {
    fn from(val: &ScrapKey) -> Self {
        val.ctx.clone()
    }
}

impl ScrapKey {
    /// Construct a key from a title and an optional hierarchical context.
    pub fn new(title: &Title, ctx: &Option<Ctx>) -> ScrapKey {
        ScrapKey {
            title: title.clone(),
            ctx: ctx.clone(),
        }
    }

    /// Convenience constructor for a single-level context.
    pub fn with_ctx(title: &Title, ctx: &Ctx) -> ScrapKey {
        ScrapKey {
            title: title.clone(),
            ctx: Some(ctx.clone()),
        }
    }

    pub fn title(&self) -> &Title {
        &self.title
    }

    pub fn ctx(&self) -> &Option<Ctx> {
        &self.ctx
    }

    /// Parse a `/`-separated path. The last non-empty segment becomes the
    /// title; the segments before it (if any) become the context. An empty
    /// input yields an empty title at the root.
    pub fn from_path_str(path: &str) -> ScrapKey {
        let mut parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        match parts.pop() {
            Some(title) => {
                let ctx = if parts.is_empty() {
                    None
                } else {
                    Some(Ctx::from(parts.join("/").as_str()))
                };
                ScrapKey {
                    title: Title::from(title),
                    ctx,
                }
            }
            None => ScrapKey::from(Title::from("")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // v1 shape:
    //   ScrapKey { title: Title, ctx: Option<Ctx> }
    // where None = root scrap, Some(ctx) = ctx is a multi-segment hierarchical
    // path (depth >= 1).
    #[rstest]
    #[case::title_only("title", "title", None)]
    #[case::single_ctx("ctx/title", "title", Some("ctx"))]
    #[case::two_levels("a/b/title", "title", Some("a/b"))]
    #[case::three_levels("a/b/c/title", "title", Some("a/b/c"))]
    #[case::four_levels("a/b/c/d/title", "title", Some("a/b/c/d"))]
    #[case::empty_path("", "", None)]
    #[case::trailing_slash_ignored("ctx/title/", "title", Some("ctx"))]
    #[case::leading_slash_ignored("/ctx/title", "title", Some("ctx"))]
    #[case::double_slash_collapsed("a//b/title", "title", Some("a/b"))]
    #[case::japanese("日本語/タイトル", "タイトル", Some("日本語"))]
    #[case::emoji("🚀/title", "title", Some("🚀"))]
    #[case::space_in_segment(
        "Book/Test driven development",
        "Test driven development",
        Some("Book")
    )]
    fn it_from_path_str(
        #[case] path: &str,
        #[case] expected_title: &str,
        #[case] expected_ctx: Option<&str>,
    ) {
        let key = ScrapKey::from_path_str(path);
        assert_eq!(key.title(), &Title::from(expected_title));
        let actual = key.ctx().as_ref().map(|c| format!("{}", c));
        assert_eq!(actual.as_deref(), expected_ctx);
    }

    #[test]
    fn it_display_root_scrap() {
        let key = ScrapKey::from(Title::from("foo"));
        assert_eq!(format!("{}", key), "foo");
    }

    #[test]
    fn it_display_single_ctx_scrap() {
        let key = ScrapKey::new(&"foo".into(), &Some("Book".into()));
        assert_eq!(format!("{}", key), "Book/foo");
    }

    #[test]
    fn it_display_nested_ctx_scrap() {
        let key = ScrapKey::new(&"foo".into(), &Some("a/b/c".into()));
        assert_eq!(format!("{}", key), "a/b/c/foo");
    }

    #[test]
    fn it_with_ctx_is_single_level() {
        let key = ScrapKey::with_ctx(&"foo".into(), &"Book".into());
        assert_eq!(format!("{}", key), "Book/foo");
        assert_eq!(key.ctx().as_ref().map(|c| c.depth()), Some(1));
    }

    #[test]
    fn it_eq_and_hash_consider_full_path() {
        use std::collections::HashSet;
        let a = ScrapKey::new(&"foo".into(), &Some("x/y".into()));
        let b = ScrapKey::new(&"foo".into(), &Some("x/y".into()));
        let c = ScrapKey::new(&"foo".into(), &Some("x/z".into()));
        let d = ScrapKey::new(&"bar".into(), &Some("x/y".into()));
        let e = ScrapKey::new(&"foo".into(), &None);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(a, e);

        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
        assert!(!set.contains(&d));
        assert!(!set.contains(&e));
    }

    #[test]
    fn it_round_trip_path_str() {
        for input in ["foo", "ctx/foo", "a/b/foo", "a/b/c/foo"] {
            let key = ScrapKey::from_path_str(input);
            assert_eq!(format!("{}", key), input);
        }
    }

    #[test]
    fn it_from_title_yields_root_scrap() {
        let key: ScrapKey = Title::from("foo").into();
        assert_eq!(key.title(), &Title::from("foo"));
        assert!(key.ctx().is_none());
    }

    #[test]
    fn it_into_title_from_value_and_ref() {
        let key = ScrapKey::new(&"foo".into(), &Some("Book".into()));
        let by_value: Title = key.clone().into();
        assert_eq!(by_value, "foo".into());
        let by_ref: Title = (&key).into();
        assert_eq!(by_ref, "foo".into());
    }

    #[test]
    fn it_into_option_ctx_from_value_and_ref() {
        let key = ScrapKey::new(&"foo".into(), &Some("a/b".into()));
        let by_value: Option<Ctx> = key.clone().into();
        assert_eq!(
            by_value.as_ref().map(|c| format!("{}", c)).as_deref(),
            Some("a/b")
        );
        let by_ref: Option<Ctx> = (&key).into();
        assert_eq!(
            by_ref.as_ref().map(|c| format!("{}", c)).as_deref(),
            Some("a/b")
        );

        let root_key = ScrapKey::from(Title::from("bar"));
        let root_ctx: Option<Ctx> = (&root_key).into();
        assert!(root_ctx.is_none());
    }
}
