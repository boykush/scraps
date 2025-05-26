use url::Url;

use crate::markdown;

use super::{context::Ctx, link::ScrapLink, title::Title};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
    pub thumbnail: Option<Url>,
}

impl Scrap {
    pub fn self_link(&self) -> ScrapLink {
        ScrapLink::new(&self.title, &self.ctx)
    }

    pub fn links(&self) -> Vec<ScrapLink> {
        markdown::extract::scrap_links(&self.md_text)
    }
}

impl Scrap {
    pub fn new(title: &str, ctx: &Option<&str>, text: &str) -> Scrap {
        let thumbnail = markdown::extract::head_image(text);

        Scrap {
            title: title.into(),
            ctx: ctx.map(|s| s.into()),
            md_text: text.to_string(),
            thumbnail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let scrap = Scrap::new("scrap title", &None, "[[link1]][[link2]][[Context/link3]]");
        assert_eq!(scrap.title, "scrap title".into());
        let mut links = scrap.links();
        links.sort();
        let mut expected = [
            Title::from("link1").into(),
            Title::from("link2").into(),
            ScrapLink::with_ctx(&"link3".into(), &"Context".into()),
        ];
        expected.sort();
        assert_eq!(links, expected);
        assert_eq!(scrap.thumbnail, None);
    }
}
