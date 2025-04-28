use url::Url;

use crate::markdown;

use super::{content::Content, context::Ctx, link::ScrapLink, title::Title};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub links: Vec<ScrapLink>,
    pub content: Content,
    pub thumbnail: Option<Url>,
}

impl Scrap {
    pub fn self_link(&self) -> ScrapLink {
        ScrapLink::new(&self.title, &self.ctx)
    }
}

impl Scrap {
    pub fn new(base_url: &Url, title: &str, ctx: &Option<&str>, text: &str) -> Scrap {
        let links = markdown::extract::scrap_links(text);
        let thumbnail = markdown::extract::head_image(text);
        let html_content = markdown::convert::to_html_content(text, base_url);

        Scrap {
            title: title.into(),
            ctx: ctx.map(|s| s.into()),
            links,
            content: html_content,
            thumbnail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let mut scrap = Scrap::new(
            &base_url,
            "scrap title",
            &None,
            "[[link1]] [[link2]] [[Context/link3]]",
        );
        assert_eq!(scrap.title, "scrap title".into());
        scrap.links.sort();
        let mut expected = [
            Title::from("link1").into(),
            Title::from("link2").into(),
            ScrapLink::with_ctx(&"link3".into(), &"Context".into()),
        ];
        expected.sort();
        assert_eq!(scrap.links, expected);
        // assert_eq!(
        //     scrap.html_content,
        //     "<p>".to_string()
        //         + &[
        //             "<a href=\"http://localhost:1112/scraps/link1.html\">link1</a>",
        //             "<a href=\"http://localhost:1112/scraps/link2.html\">link2</a>",
        //             "<a href=\"http://localhost:1112/scraps/link3.context.html\">link3</a>",
        //         ]
        //         .join(" ")
        //         + "</p>\n"
        // );
        assert_eq!(scrap.thumbnail, None);
    }
}
