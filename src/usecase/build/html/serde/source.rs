use scraps_libs::model::source::Source;
use url::Url;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SourceTera {
    resource: String,
    title: Option<String>,
    /// What a reader sees: the title, or the resource when there is none.
    label: String,
    /// The resource itself when a browser can follow it; anything else
    /// (a path, a scope description) stays text and never links a scrap.
    href: Option<String>,
}

impl From<&Source> for SourceTera {
    fn from(source: &Source) -> Self {
        let href = web_href(&source.resource);
        SourceTera {
            resource: source.resource.clone(),
            title: source.title.clone(),
            label: source.title.clone().unwrap_or(source.resource.clone()),
            href,
        }
    }
}

/// What a scrap is about, as its top-level `resource` names it.
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct ResourceTera {
    resource: String,
    href: Option<String>,
}

impl ResourceTera {
    pub fn new(resource: &str) -> Self {
        ResourceTera {
            resource: resource.to_string(),
            href: web_href(resource),
        }
    }
}

fn web_href(resource: &str) -> Option<String> {
    Url::parse(resource)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https"))
        .map(|_| resource.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn source(resource: &str) -> Source {
        Source {
            resource: resource.to_string(),
            id: None,
            title: None,
            author: None,
            last_modified: None,
        }
    }

    #[test]
    fn it_labels_a_source_by_its_title_or_else_its_resource() {
        let titled = Source {
            title: Some("Handbook".to_string()),
            ..source("https://example.com")
        };
        assert_eq!(SourceTera::from(&titled).label, "Handbook");
        assert_eq!(
            SourceTera::from(&source("https://example.com")).label,
            "https://example.com"
        );
    }

    #[rstest]
    #[case::https("https://example.com/handbook")]
    #[case::http("http://example.com")]
    fn it_links_a_web_url(#[case] resource: &str) {
        assert_eq!(
            SourceTera::from(&source(resource)).href.as_deref(),
            Some(resource)
        );
    }

    #[rstest]
    #[case::relative_path("references/ga4-schema.md")]
    #[case::scrap_key("Book/Domain-Driven Design")]
    #[case::description("all queries in BigQuery project X")]
    #[case::script("javascript:alert(1)")]
    fn it_keeps_anything_else_as_text(#[case] resource: &str) {
        assert_eq!(SourceTera::from(&source(resource)).href, None);
    }

    #[rstest]
    #[case::web("https://example.com/a", Some("https://example.com/a"))]
    #[case::other_scheme("bigquery://acme/sales/orders", None)]
    fn it_links_a_resource_only_when_a_browser_can_follow_it(
        #[case] resource: &str,
        #[case] href: Option<&str>,
    ) {
        assert_eq!(ResourceTera::new(resource).href.as_deref(), href);
    }
}
