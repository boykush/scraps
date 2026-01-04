use std::fmt::Display;

use crate::slugify;

use super::{context::Ctx, title::Title};

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Slug(String);

impl From<Title> for Slug {
    fn from(title: Title) -> Self {
        Slug(slugify::by_dash(&title.to_string()))
    }
}

impl From<Ctx> for Slug {
    fn from(ctx: Ctx) -> Self {
        Slug(slugify::by_dash(&ctx.to_string()))
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::basic("Hello World", "hello-world")]
    #[case::special_chars("Hello, World!", "hello-comma-world-exclamation")]
    #[case::unicode("こんにちは世界", "こんにちは世界")]
    #[case::numbers("Test 123 Article", "test-123-article")]
    #[case::empty("", "")]
    #[case::multiple_spaces("Multiple   Spaces   Here", "multiple-spaces-here")]
    #[case::leading_trailing_spaces("  Leading and Trailing  ", "leading-and-trailing")]
    fn test_slug_from_title(#[case] input: &str, #[case] expected: &str) {
        let title = Title::from(input);
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), expected);
    }

    #[rstest]
    #[case::basic("Hello Context", "hello-context")]
    #[case::special_chars("Hello/Context@Test", "hello-slash-context-at-test")]
    #[case::unicode("テストコンテキスト", "テストコンテキスト")]
    #[case::empty("", "")]
    fn test_slug_from_ctx(#[case] input: &str, #[case] expected: &str) {
        let ctx = Ctx::from(input);
        let slug = Slug::from(ctx);
        assert_eq!(slug.to_string(), expected);
    }

    #[test]
    fn test_slug_display() {
        let slug = Slug("test-slug".to_string());
        assert_eq!(format!("{}", slug), "test-slug");
    }
}
