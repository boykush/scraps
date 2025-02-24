use url::Url;

use crate::markdown;

use super::title::Title;

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: Title,
    pub links: Vec<Title>,
    pub html_content: String,
    pub thumbnail: Option<Url>,
}

impl Scrap {
    pub fn new(base_url: &Url, title: &str, text: &str) -> Scrap {
        let links = markdown::extract::link_titles(text)
            .iter()
            .map(|t| t.as_str().into())
            .collect();
        let thumbnail = markdown::extract::head_image(text);
        let html_content = markdown::to_html(text, base_url);

        Scrap {
            title: title.into(),
            links,
            html_content,
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
        let mut scrap = Scrap::new(&base_url, "scrap title", "[[link1]] [[link2]]");
        assert_eq!(scrap.title, "scrap title".into());
        scrap.links.sort();
        let mut expected = ["link1".into(), "link2".into()];
        expected.sort();
        assert_eq!(scrap.links, expected);
        assert_eq!(
            scrap.html_content,
            "<p><a href=\"http://localhost:1112/scraps/link1.html\">link1</a> <a href=\"http://localhost:1112/scraps/link2.html\">link2</a></p>\n"
                .to_string()
        );
        assert_eq!(scrap.thumbnail, None);
    }
}
