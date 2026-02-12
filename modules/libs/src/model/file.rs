use std::fmt::Display;

use super::{context::Ctx, key::ScrapKey, slug::Slug, title::Title};

pub struct ScrapFileStem(String);

impl From<ScrapKey> for ScrapFileStem {
    fn from(key: ScrapKey) -> Self {
        let title: Title = Title::from(&key);
        let ctx: Option<Ctx> = Option::<Ctx>::from(&key);
        let file_name = match ctx {
            Some(ctx) => format!("{}.{}", Slug::from(title), Slug::from(ctx)),
            None => Slug::from(title).to_string(),
        };
        ScrapFileStem(file_name)
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

    #[rstest]
    #[case::simple_title(ScrapKey::from(Title::from("title")), "title")]
    #[case::slugified_title(ScrapKey::from(Title::from("expected slugify")), "expected-slugify")]
    #[case::with_context(ScrapKey::with_ctx(&"title".into(), &"Context".into()), "title.context")]
    fn it_from_scrap_link(#[case] input: ScrapKey, #[case] expected: &str) {
        let file_name = ScrapFileStem::from(input);
        assert_eq!(file_name.to_string(), expected);
    }
}
