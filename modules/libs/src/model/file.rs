use std::fmt::Display;

use super::{link::ScrapLink, slug::Slug};

pub struct HtmlFileName(String);

impl From<ScrapLink> for HtmlFileName {
    fn from(link: ScrapLink) -> Self {
        let file_name = match link.ctx {
            Some(ctx) => format!("{}.{}.html", Slug::from(link.title), Slug::from(ctx)),
            None => format!("{}.html", Slug::from(link.title)),
        };
        HtmlFileName(file_name)
    }
}

impl Display for HtmlFileName {
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
        let expected_list = ["title.html", "expected-slugify.html", "title.context.html"];
        input_list
            .iter()
            .zip(expected_list.iter())
            .for_each(|(input, expected)| {
                let file_name = HtmlFileName::from(input.clone());
                assert_eq!(file_name.to_string(), *expected);
            });
    }
}
