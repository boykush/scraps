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

    use super::*;

    #[test]
    fn it_from_scrap_link() {
        let input_list = [
            ScrapKey::from(Title::from("title")),
            ScrapKey::from(Title::from("expected slugify")),
            ScrapKey::with_ctx(&"title".into(), &"Context".into()),
        ];
        let expected_list = ["title", "expected-slugify", "title.context"];
        input_list
            .iter()
            .zip(expected_list.iter())
            .for_each(|(input, expected)| {
                let file_name = ScrapFileStem::from(input.clone());
                assert_eq!(file_name.to_string(), *expected);
            });
    }
}
