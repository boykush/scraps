use url::Url;

use crate::libs::markdown;

use super::title::Title;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let mut scrap = Scrap::new("scrap title", "[[link1]] [[link2]]", &None);
        assert_eq!(scrap.title, Title::new("scrap title"));
        assert_eq!(scrap.links.sort(), vec!(Title::new("link1"), Title::new("link2")).sort());
        assert_eq!(scrap.html_content, "<p><a href=\"./link1.html\">link1</a> <a href=\"./link2.html\">link2</a></p>\n"
                        .to_string());
        assert_eq!(scrap.thumbnail, None);
        assert_eq!(scrap.commited_ts, None);
    }
}
