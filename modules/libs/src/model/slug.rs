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

    #[test]
    fn test_slug_from_title() {
        let title = Title::from("Hello World");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "hello-world");
    }

    #[test]
    fn test_slug_from_title_with_special_chars() {
        let title = Title::from("Hello, World!");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "hello-comma-world-exclamation");
    }

    #[test]
    fn test_slug_from_title_with_unicode() {
        let title = Title::from("こんにちは世界");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "こんにちは世界");
    }

    #[test]
    fn test_slug_from_title_with_numbers() {
        let title = Title::from("Test 123 Article");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "test-123-article");
    }

    #[test]
    fn test_slug_from_ctx() {
        let ctx = Ctx::from("Hello Context");
        let slug = Slug::from(ctx);
        assert_eq!(slug.to_string(), "hello-context");
    }

    #[test]
    fn test_slug_from_ctx_with_special_chars() {
        let ctx = Ctx::from("Hello/Context@Test");
        let slug = Slug::from(ctx);
        assert_eq!(slug.to_string(), "hello-slash-context-at-test");
    }

    #[test]
    fn test_slug_from_ctx_with_unicode() {
        let ctx = Ctx::from("テストコンテキスト");
        let slug = Slug::from(ctx);
        assert_eq!(slug.to_string(), "テストコンテキスト");
    }

    #[test]
    fn test_slug_display() {
        let slug = Slug("test-slug".to_string());
        assert_eq!(format!("{}", slug), "test-slug");
    }

    #[test]
    fn test_slug_from_empty_title() {
        let title = Title::from("");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "");
    }

    #[test]
    fn test_slug_from_empty_ctx() {
        let ctx = Ctx::from("");
        let slug = Slug::from(ctx);
        assert_eq!(slug.to_string(), "");
    }

    #[test]
    fn test_slug_with_multiple_spaces() {
        let title = Title::from("Multiple   Spaces   Here");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "multiple-spaces-here");
    }

    #[test]
    fn test_slug_with_leading_trailing_spaces() {
        let title = Title::from("  Leading and Trailing  ");
        let slug = Slug::from(title);
        assert_eq!(slug.to_string(), "leading-and-trailing");
    }
}
