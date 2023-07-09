use url::Url;

use crate::libs::markdown;

#[derive(PartialEq, Clone, Debug)]
pub struct Scrap {
    pub title: String,
    pub links: Vec<String>,
    pub html_content: String,
    pub thumbnail: Option<Url>,
    pub updated_ts: u64,
}

impl Scrap {
    pub fn new(title: &str, text: &str, updated_ts: &u64) -> Scrap {
        let links = markdown::extract_link_titles(text);
        let thumbnail = markdown::head_image(text);
        let html_content = markdown::to_html(text);

        Scrap {
            title: title.to_string(),
            links: links,
            html_content: html_content,
            thumbnail: thumbnail,
            updated_ts: updated_ts.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new() {
        let scrap1 = Scrap::new("scrap1", "[[link1]] ![](https://example.com/image.png)", &0);
        assert_eq!(
            scrap1,
            Scrap {
                title: "scrap1".to_string(),
                links: vec!("link1".to_string()),
                html_content: "<p><a href=\"./link1.html\">link1</a> <img src=\"https://example.com/image.png\" alt=\"\" /></p>\n".to_string(),
                thumbnail: Some(Url::parse("https://example.com/image.png").unwrap()),
                updated_ts: 0
            }
        );

        let scrap2 = Scrap::new("scrap2", "[[link1]] [[link2]]", &0);
        assert_eq!(
            scrap2,
            Scrap {
                title: "scrap2".to_string(),
                links: vec!("link1".to_string(), "link2".to_string()),
                html_content:
                    "<p><a href=\"./link1.html\">link1</a> <a href=\"./link2.html\">link2</a></p>\n"
                        .to_string(),
                thumbnail: None,
                updated_ts: 0
            }
        )
    }
}
