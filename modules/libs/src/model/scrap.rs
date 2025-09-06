use url::Url;

use crate::markdown;

use super::{context::Ctx, key::ScrapKey, title::Title};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub links: Vec<ScrapKey>,
    pub md_text: String,
    pub thumbnail: Option<Url>,
}

impl Scrap {
    pub fn self_link(&self) -> ScrapKey {
        ScrapKey::new(&self.title, &self.ctx)
    }
}

impl Scrap {
    pub fn new(title: &str, ctx: &Option<&str>, text: &str) -> Scrap {
        let links = markdown::extract::scrap_links(text);
        let thumbnail = markdown::extract::head_image(text);

        Scrap {
            title: title.into(),
            ctx: ctx.map(|s| s.into()),
            links,
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
        let mut scrap = Scrap::new("scrap title", &None, "[[link1]][[link2]][[Context/link3]]");
        assert_eq!(scrap.title, "scrap title".into());
        scrap.links.sort();
        let mut expected = [
            Title::from("link1").into(),
            Title::from("link2").into(),
            ScrapKey::with_ctx(&"link3".into(), &"Context".into()),
        ];
        expected.sort();
        assert_eq!(scrap.links, expected);
        assert_eq!(scrap.thumbnail, None);
    }
}
