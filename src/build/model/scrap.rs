use std::fmt::Display;

use url::Url;

use crate::libs::{markdown, slugify};

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: Title,
    pub slug: Slug,
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
            slug: Slug::new(title),
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

#[derive(PartialEq, Clone, Debug)]
pub struct Slug(String);

impl Slug {
    pub fn new(v: &str) -> Slug {
        let slug = slugify::by_dash(v);
        Slug(slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let mut scrap = Scrap::new("scrap title", "[[link1]] [[link2]]", &None);
        assert_eq!(scrap.title, Title::new("scrap title"));
        assert_eq!(scrap.slug, Slug::new("scrap-title"));
        assert_eq!(scrap.links.sort(), vec!(Title::new("link1"), Title::new("link2")).sort());
        assert_eq!(scrap.html_content, "<p><a href=\"./link1.html\">link1</a> <a href=\"./link2.html\">link2</a></p>\n"
                        .to_string());
        assert_eq!(scrap.thumbnail, None);
        assert_eq!(scrap.commited_ts, None);
    }
}
