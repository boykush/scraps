use std::fmt::Display;

use super::{link::ScrapLink, slug::Slug};

pub struct ScrapFileStem(String);

impl From<ScrapLink> for ScrapFileStem {
    fn from(link: ScrapLink) -> Self {
        let file_name = match link.ctx {
            Some(ctx) => format!("{}.{}", Slug::from(link.title), Slug::from(ctx)),
            None => Slug::from(link.title).to_string(),
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
            ScrapLink::from(Title::from("title")),
            ScrapLink::from(Title::from("expected slugify")),
            ScrapLink::with_ctx(&"title".into(), &"Context".into()),
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
