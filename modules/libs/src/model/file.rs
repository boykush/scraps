use std::fmt::Display;

use crate::slugify;

use super::{context::Ctx, key::ScrapKey, slug::Slug, title::Title};

/// Path-shaped stem for a scrap, using slash-separated segments. Each segment
/// (context segments and the title) is slugified independently. Examples:
///   - root scrap "Title"       → "title"
///   - ctx "Context"            → "context/title"
///   - ctx "Programming/Rust"   → "programming/rust/title"
pub struct ScrapFileStem(String);

impl From<ScrapKey> for ScrapFileStem {
    fn from(key: ScrapKey) -> Self {
        let title: Title = Title::from(&key);
        let ctx: Option<Ctx> = Option::<Ctx>::from(&key);
        let title_slug = Slug::from(title).to_string();

        let stem = match ctx {
            Some(ctx) => {
                let ctx_slug_path: Vec<String> = ctx
                    .segments()
                    .iter()
                    .map(|seg| slugify::by_dash(seg))
                    .collect();
                format!("{}/{}", ctx_slug_path.join("/"), title_slug)
            }
            None => title_slug,
        };
        ScrapFileStem(stem)
    }
}

impl Display for ScrapFileStem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::title::Title;
    use rstest::rstest;

    use super::*;

    // v1 shape: ScrapFileStem maps to a nested directory layout.
    //   <root scrap>          → "title"
    //   ctx = "Context"       → "context/title"
    //   ctx = "a/b"           → "a/b/title"
    //   each ctx segment is slugified independently
    #[rstest]
    #[case::simple_title(ScrapKey::from(Title::from("title")), "title")]
    #[case::slugified_title(ScrapKey::from(Title::from("expected slugify")), "expected-slugify")]
    #[case::single_level_ctx(
        ScrapKey::with_ctx(&"title".into(), &"Context".into()),
        "context/title"
    )]
    #[case::two_level_ctx(
        ScrapKey::new(&"borrowing".into(), &Some("Programming/Rust".into())),
        "programming/rust/borrowing"
    )]
    #[case::three_level_ctx(
        ScrapKey::new(&"foo".into(), &Some("a/b/c".into())),
        "a/b/c/foo"
    )]
    #[case::slugify_per_segment(
        ScrapKey::new(&"My Title".into(), &Some("Outer Group/Inner Group".into())),
        "outer-group/inner-group/my-title"
    )]
    #[case::unicode_segments(
        ScrapKey::new(&"borrowing".into(), &Some("プログラミング/Rust".into())),
        "プログラミング/rust/borrowing"
    )]
    fn it_from_scrap_link(#[case] input: ScrapKey, #[case] expected: &str) {
        let file_name = ScrapFileStem::from(input);
        assert_eq!(file_name.to_string(), expected);
    }
}
