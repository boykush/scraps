use url::Url;

use crate::markdown;

use super::{context::Ctx, key::ScrapKey, title::Title};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    title: Title,
    ctx: Option<Ctx>,
    links: Vec<ScrapKey>,
    md_text: String,
    thumbnail: Option<Url>,
}

impl Scrap {
    pub fn self_key(&self) -> ScrapKey {
        ScrapKey::new(&self.title, &self.ctx)
    }

    pub fn title(&self) -> &Title {
        &self.title
    }

    pub fn ctx(&self) -> &Option<Ctx> {
        &self.ctx
    }

    pub fn links(&self) -> &[ScrapKey] {
        &self.links
    }

    pub fn md_text(&self) -> &str {
        &self.md_text
    }

    pub fn thumbnail(&self) -> Option<Url> {
        self.thumbnail.clone()
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
        let scrap = Scrap::new("scrap title", &None, "[[link1]][[link2]][[Context/link3]]");
        assert_eq!(scrap.title(), &"scrap title".into());

        let mut actual_links = scrap.links().to_vec();
        actual_links.sort();

        let mut expected = [
            Title::from("link1").into(),
            Title::from("link2").into(),
            ScrapKey::with_ctx(&"link3".into(), &"Context".into()),
        ];
        expected.sort();

        assert_eq!(actual_links, expected);
        assert_eq!(scrap.thumbnail(), None);
    }
}
