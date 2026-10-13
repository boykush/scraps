use std::fmt;

use super::{context::Ctx, stem::Stem};

/// What identifies a scrap and what links point at: its file path under the
/// wiki root without `.md`, held as the file stem and the folder context.
/// `ctx == None` is a scrap at the root; `Display` writes the key as one
/// `/`-separated string, the form that goes inside `[[ ]]`.
#[derive(PartialEq, Clone, Debug, PartialOrd, Eq, Ord, Hash)]
pub struct ScrapKey {
    stem: Stem,
    ctx: Option<Ctx>,
}

impl From<Stem> for ScrapKey {
    fn from(stem: Stem) -> Self {
        ScrapKey { stem, ctx: None }
    }
}

impl fmt::Display for ScrapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ctx {
            Some(ctx) => write!(f, "{}/{}", ctx, self.stem),
            None => write!(f, "{}", self.stem),
        }
    }
}

impl From<ScrapKey> for Stem {
    fn from(val: ScrapKey) -> Self {
        val.stem
    }
}

impl From<&ScrapKey> for Stem {
    fn from(val: &ScrapKey) -> Self {
        val.stem.clone()
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
    /// Construct a key from a stem and an optional hierarchical context.
    pub fn new(stem: &Stem, ctx: &Option<Ctx>) -> ScrapKey {
        ScrapKey {
            stem: stem.clone(),
            ctx: ctx.clone(),
        }
    }

    /// Convenience constructor for a single-level context.
    pub fn with_ctx(stem: &Stem, ctx: &Ctx) -> ScrapKey {
        ScrapKey {
            stem: stem.clone(),
            ctx: Some(ctx.clone()),
        }
    }

    pub fn stem(&self) -> &Stem {
        &self.stem
    }

    pub fn ctx(&self) -> &Option<Ctx> {
        &self.ctx
    }

    /// Parse a `/`-separated path. The last non-empty segment becomes the
    /// stem; the segments before it (if any) become the context. An empty
    /// input yields an empty stem at the root.
    pub fn from_path_str(path: &str) -> ScrapKey {
        let mut parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        match parts.pop() {
            Some(stem) => {
                let ctx = if parts.is_empty() {
                    None
                } else {
                    Some(Ctx::from(parts.join("/").as_str()))
                };
                ScrapKey {
                    stem: Stem::from(stem),
                    ctx,
                }
            }
            None => ScrapKey::from(Stem::from("")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // v1 shape:
    //   ScrapKey { stem: Stem, ctx: Option<Ctx> }
    // where None = root scrap, Some(ctx) = ctx is a multi-segment hierarchical
    // path (depth >= 1).
    #[rstest]
    #[case::stem_only("stem", "stem", None)]
    #[case::single_ctx("ctx/stem", "stem", Some("ctx"))]
    #[case::two_levels("a/b/stem", "stem", Some("a/b"))]
    #[case::three_levels("a/b/c/stem", "stem", Some("a/b/c"))]
    #[case::four_levels("a/b/c/d/stem", "stem", Some("a/b/c/d"))]
    #[case::empty_path("", "", None)]
    #[case::trailing_slash_ignored("ctx/stem/", "stem", Some("ctx"))]
    #[case::leading_slash_ignored("/ctx/stem", "stem", Some("ctx"))]
    #[case::double_slash_collapsed("a//b/stem", "stem", Some("a/b"))]
    #[case::japanese("日本語/タイトル", "タイトル", Some("日本語"))]
    #[case::emoji("🚀/stem", "stem", Some("🚀"))]
    #[case::space_in_segment(
        "Book/Test driven development",
        "Test driven development",
        Some("Book")
    )]
    fn it_from_path_str(
        #[case] path: &str,
        #[case] expected_stem: &str,
        #[case] expected_ctx: Option<&str>,
    ) {
        let key = ScrapKey::from_path_str(path);
        assert_eq!(key.stem(), &Stem::from(expected_stem));
        let actual = key.ctx().as_ref().map(|c| format!("{}", c));
        assert_eq!(actual.as_deref(), expected_ctx);
    }

    #[test]
    fn it_display_root_scrap() {
        let key = ScrapKey::from(Stem::from("foo"));
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
    fn it_from_stem_yields_root_scrap() {
        let key: ScrapKey = Stem::from("foo").into();
        assert_eq!(key.stem(), &Stem::from("foo"));
        assert!(key.ctx().is_none());
    }

    #[test]
    fn it_into_stem_from_value_and_ref() {
        let key = ScrapKey::new(&"foo".into(), &Some("Book".into()));
        let by_value: Stem = key.clone().into();
        assert_eq!(by_value, "foo".into());
        let by_ref: Stem = (&key).into();
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

        let root_key = ScrapKey::from(Stem::from("bar"));
        let root_ctx: Option<Ctx> = (&root_key).into();
        assert!(root_ctx.is_none());
    }
}
