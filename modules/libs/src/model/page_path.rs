use std::fmt::{Display, Write};

use super::{base_url::BaseUrl, key::ScrapKey, tag::Tag};

/// Where the static site writes a page, relative to the output directory and
/// `/`-separated. The path is the name as written, never a slug of it, so two
/// names share a path exactly when the model compares them equal.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct PagePath(String);

impl PagePath {
    /// A key comes from a file's own path, so it is used untouched: sources
    /// the filesystem keeps apart stay apart as pages.
    pub fn of_scrap(key: &ScrapKey) -> PagePath {
        PagePath(format!("scraps/{key}.html"))
    }

    /// A nested tag keeps its hierarchy as directories.
    pub fn of_tag(tag: &Tag) -> PagePath {
        let segments: Vec<String> = tag.segments().iter().map(|s| name_segment(s)).collect();
        PagePath(format!("tags/{}.html", segments.join("/")))
    }

    /// The directory of the list views narrowed to one type.
    pub fn of_type(name: &str) -> PagePath {
        PagePath(format!("types/{}/", name_segment(name)))
    }

    /// The URL a browser requests for this page.
    pub fn to_url(&self, base_url: &BaseUrl) -> String {
        format!("{}{}", base_url.as_url(), self.to_url_path())
    }

    /// The URL relative to the site's base URL: each segment percent-encoded
    /// down to RFC 3986 unreserved characters, `/` kept as the separator.
    pub fn to_url_path(&self) -> String {
        let mut url = String::with_capacity(self.0.len());
        for (i, segment) in self.0.split('/').enumerate() {
            if i > 0 {
                url.push('/');
            }
            for byte in segment.bytes() {
                match byte {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                        url.push(byte as char)
                    }
                    _ => write!(url, "%{byte:02X}").expect("writing to a String cannot fail"),
                }
            }
        }
        url
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PagePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// A tag segment or a type is free text, not a file name, so three spellings
// cannot be used as written: `/` would split the name, `.` and `..` would
// leave its directory, and `%` has to be escaped for the escapes to stay
// injective. Everything else, case included, is kept.
fn name_segment(name: &str) -> String {
    if name == "." || name == ".." {
        return name.replace('.', "%2E");
    }
    name.replace('%', "%25").replace('/', "%2F")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use url::Url;

    use super::*;

    fn base_url() -> BaseUrl {
        BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap()
    }

    #[rstest]
    #[case::root("title", "scraps/title.html")]
    #[case::case_and_spaces_kept("Reference/CLI Overview", "scraps/Reference/CLI Overview.html")]
    #[case::nested_ctx("a/b/c/foo", "scraps/a/b/c/foo.html")]
    #[case::reserved_characters("What is it?", "scraps/What is it?.html")]
    #[case::percent("100% sure", "scraps/100% sure.html")]
    #[case::unicode("プログラミング/Rust/所有権", "scraps/プログラミング/Rust/所有権.html")]
    fn it_writes_a_scrap_at_its_key(#[case] key: &str, #[case] expected: &str) {
        let path = PagePath::of_scrap(&ScrapKey::from_path_str(key));
        assert_eq!(path.as_str(), expected);
    }

    #[rstest]
    #[case::whitespace_and_dash("a b", "a-b")]
    #[case::case("Blog", "blog")]
    #[case::reserved_word("What?", "What question")]
    #[case::ctx_and_title("a/b", "a-b")]
    fn it_keeps_distinct_keys_on_distinct_paths(#[case] a: &str, #[case] b: &str) {
        assert_ne!(
            PagePath::of_scrap(&ScrapKey::from_path_str(a)),
            PagePath::of_scrap(&ScrapKey::from_path_str(b))
        );
    }

    #[rstest]
    #[case::flat("Domain Driven Design", "tags/Domain Driven Design.html")]
    #[case::nested("AI/ML", "tags/AI/ML.html")]
    #[case::named_like_a_listing("index", "tags/index.html")]
    #[case::percent("100%", "tags/100%25.html")]
    #[case::parent_dir("../index", "tags/%2E%2E/index.html")]
    #[case::current_dir("./a", "tags/%2E/a.html")]
    fn it_writes_a_tag_at_its_name(#[case] tag: &str, #[case] expected: &str) {
        assert_eq!(PagePath::of_tag(&Tag::from(tag)).as_str(), expected);
    }

    #[test]
    fn it_shares_a_tag_path_only_between_equal_tags() {
        // Empty segments are dropped by the model, so these are one tag.
        assert_eq!(Tag::from("a//b/"), Tag::from("a/b"));
        assert_eq!(
            PagePath::of_tag(&Tag::from("a//b/")),
            PagePath::of_tag(&Tag::from("a/b"))
        );

        for (a, b) in [
            ("a b", "a-b"),
            ("Rust", "rust"),
            ("..", "%2E%2E"),
            ("%", "%25"),
        ] {
            assert_ne!(Tag::from(a), Tag::from(b));
            assert_ne!(
                PagePath::of_tag(&Tag::from(a)),
                PagePath::of_tag(&Tag::from(b))
            );
        }
    }

    #[rstest]
    #[case::as_written("BigQuery Table", "types/BigQuery Table/")]
    #[case::slash("ADR/Accepted", "types/ADR%2FAccepted/")]
    #[case::percent("a%2Fb", "types/a%252Fb/")]
    #[case::parent_dir("..", "types/%2E%2E/")]
    #[case::dash("-", "types/-/")]
    fn it_writes_a_type_under_its_name(#[case] name: &str, #[case] expected: &str) {
        assert_eq!(PagePath::of_type(name).as_str(), expected);
    }

    #[rstest]
    #[case::case("Blog", "blog")]
    #[case::former_suffix("blog", "blog-2")]
    #[case::slash_and_its_escape("a/b", "a%2Fb")]
    #[case::dot_and_its_escape(".", "%2E")]
    fn it_keeps_distinct_types_on_distinct_paths(#[case] a: &str, #[case] b: &str) {
        assert_ne!(PagePath::of_type(a), PagePath::of_type(b));
    }

    #[rstest]
    #[case::plain("scraps/title.html", "http://localhost:1112/scraps/title.html")]
    #[case::space(
        "scraps/Reference/CLI Overview.html",
        "http://localhost:1112/scraps/Reference/CLI%20Overview.html"
    )]
    #[case::question(
        "scraps/What is it?.html",
        "http://localhost:1112/scraps/What%20is%20it%3F.html"
    )]
    #[case::hash("scraps/C#.html", "http://localhost:1112/scraps/C%23.html")]
    #[case::pipe("scraps/a|b.html", "http://localhost:1112/scraps/a%7Cb.html")]
    #[case::percent(
        "scraps/100% sure.html",
        "http://localhost:1112/scraps/100%25%20sure.html"
    )]
    #[case::unicode(
        "scraps/日本語.html",
        "http://localhost:1112/scraps/%E6%97%A5%E6%9C%AC%E8%AA%9E.html"
    )]
    #[case::escaped_on_disk(
        "types/ADR%2FAccepted/",
        "http://localhost:1112/types/ADR%252FAccepted/"
    )]
    fn it_percent_encodes_each_segment_of_the_url(#[case] path: &str, #[case] expected: &str) {
        assert_eq!(PagePath(path.to_string()).to_url(&base_url()), expected);
    }

    #[test]
    fn it_puts_the_url_under_a_base_url_with_a_path() {
        let base_url = BaseUrl::new(Url::parse("https://example.com/wiki").unwrap()).unwrap();
        let path = PagePath::of_scrap(&ScrapKey::from_path_str("Guide/Set up"));
        assert_eq!(
            path.to_url(&base_url),
            "https://example.com/wiki/scraps/Guide/Set%20up.html"
        );
    }
}
