use std::fmt::Display;

use url::Url;

use crate::libs::markdown;

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: Title,
    pub links: Vec<Title>,
    pub html_content: String,
    pub thumbnail: Option<Url>,
    pub commited_ts: Option<i64>,
}

impl Scrap {
    pub fn new(title: &str, text: &str, commited_ts: &Option<i64>) -> Scrap {
        let links = markdown::extract_link_titles(text)
            .iter()
            .map(|t| Title::new(t))
            .collect();
        let thumbnail = markdown::head_image(text);
        let html_content = markdown::to_html(text);

        Scrap {
            title: Title::new(title),
            links: links,
            html_content: html_content,
            thumbnail: thumbnail,
            commited_ts: commited_ts.to_owned(),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Title(String);

impl Title {
    pub fn new(title: &str) -> Title {
        Title(title.to_owned())
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let scrap1 = Scrap::new(
            "scrap1",
            "[[link1]] ![](https://example.com/image.png)",
            &None,
        );
        assert_eq!(
            scrap1,
            Scrap {
                title: Title::new("scrap1"),
                links: vec!(Title::new("link1")),
                html_content: "<p><a href=\"./link1.html\">link1</a> <img src=\"https://example.com/image.png\" alt=\"\" /></p>\n".to_string(),
                thumbnail: Some(Url::parse("https://example.com/image.png").unwrap()),
                commited_ts: None
            }
        );

        let mut scrap2 = Scrap::new("scrap2", "[[link1]] [[link2]]", &None);
        assert_eq!(scrap2.title, Title::new("scrap2"));
        assert_eq!(scrap2.links.sort(), vec!(Title::new("link1"), Title::new("link2")).sort());
        assert_eq!(scrap2.html_content, "<p><a href=\"./link1.html\">link1</a> <a href=\"./link2.html\">link2</a></p>\n"
                        .to_string());
        assert_eq!(scrap2.thumbnail, None);
        assert_eq!(scrap2.commited_ts, None);
    }
}
