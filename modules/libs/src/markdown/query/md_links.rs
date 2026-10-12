use crate::model::{context::Ctx, key::ScrapKey, title::Title};

const SCRAP_EXTENSION: &str = ".md";

/// A Markdown link whose destination is the path of a `.md` file, kept as
/// written: where it lands depends on the scrap that holds it, so resolving
/// is left to [`MdLinkRef::resolve`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdLinkRef {
    /// The destination as authored, fragment included.
    pub dest: String,
    /// The link text, which plays the role of a wiki-link alias.
    pub alias: Option<String>,
    pub line: usize,
}

impl MdLinkRef {
    /// `None` unless `dest` has the shape of a path to a scrap file.
    pub(super) fn new(dest: &str, label: String, line: usize) -> Option<MdLinkRef> {
        split_scrap_dest(dest)?;
        Some(MdLinkRef {
            dest: dest.to_string(),
            alias: (!label.is_empty()).then_some(label),
            line,
        })
    }

    /// The `#fragment` of the destination, read as a heading reference.
    pub fn heading(&self) -> Option<String> {
        split_scrap_dest(&self.dest).and_then(|(_, heading)| heading)
    }

    /// The scrap this link points at when written in a scrap living in `ctx`.
    pub fn resolve(&self, ctx: &Option<Ctx>) -> Option<ScrapKey> {
        resolve_scrap_dest(&self.dest, ctx).map(|(key, _)| key)
    }

    /// The link as an author would recognise it in a message.
    pub fn written(&self) -> String {
        format!("[{}]({})", self.alias.as_deref().unwrap_or(""), self.dest)
    }
}

/// Resolve a Markdown link destination to the scrap key and heading it names:
/// a `/`-leading path from the wiki root, any other relative to `ctx`.
/// `None` when it is not a scrap path or when it climbs out of the wiki root.
pub fn resolve_scrap_dest(dest: &str, ctx: &Option<Ctx>) -> Option<(ScrapKey, Option<String>)> {
    let (path, heading) = split_scrap_dest(dest)?;
    let mut segments: Vec<&str> = match ctx {
        Some(ctx) if !path.starts_with('/') => ctx.segments().iter().map(String::as_str).collect(),
        _ => Vec::new(),
    };
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => segments.push(other),
        }
    }
    let title = segments.pop()?.strip_suffix(SCRAP_EXTENSION)?;
    if title.is_empty() {
        return None;
    }
    let ctx = (!segments.is_empty()).then(|| Ctx::from(segments.join("/").as_str()));
    Some((ScrapKey::new(&Title::from(title), &ctx), heading))
}

/// Split a destination into its percent-decoded path and heading, provided
/// it carries no scheme or host and the path ends in `.md`.
fn split_scrap_dest(dest: &str) -> Option<(String, Option<String>)> {
    if has_scheme(dest) || dest.starts_with("//") {
        return None;
    }
    let (path, fragment) = match dest.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (dest, None),
    };
    let path = percent_decode(path);
    if !path.ends_with(SCRAP_EXTENSION) {
        return None;
    }
    let heading = fragment.map(percent_decode).filter(|h| !h.is_empty());
    Some((path, heading))
}

fn has_scheme(dest: &str) -> bool {
    let Some((scheme, _)) = dest.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%')
            .then(|| s.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn resolve(dest: &str, ctx: Option<&str>) -> Option<(String, Option<String>)> {
        resolve_scrap_dest(dest, &ctx.map(Ctx::from)).map(|(key, h)| (key.to_string(), h))
    }

    #[rstest]
    #[case::absolute("/tables/customers.md", Some("metrics"), "tables/customers")]
    #[case::absolute_root("/index.md", Some("a/b"), "index")]
    #[case::bare_sibling("other.md", Some("tables"), "tables/other")]
    #[case::dot_sibling("./other.md", Some("tables"), "tables/other")]
    #[case::parent("../metrics/revenue.md", Some("tables"), "metrics/revenue")]
    #[case::parent_to_root("../top.md", Some("tables"), "top")]
    #[case::from_root("tables/orders.md", None, "tables/orders")]
    #[case::normalised("./a/./b/../c.md", None, "a/c")]
    #[case::percent_decoded(
        "Static%20Site/Pipeline.md",
        Some("Reference"),
        "Reference/Static Site/Pipeline"
    )]
    #[case::unicode_percent("/%E6%97%A5%E6%9C%AC.md", None, "日本")]
    #[case::dotted_title("/notes/v1.2.md", None, "notes/v1.2")]
    #[case::decomposed("/cafe\u{301}/cafe\u{301}.md", None, "caf\u{e9}/caf\u{e9}")]
    #[case::decomposed_percent("/cafe%CC%81.md", None, "caf\u{e9}")]
    fn it_resolves_a_scrap_path(
        #[case] dest: &str,
        #[case] ctx: Option<&str>,
        #[case] expected: &str,
    ) {
        assert_eq!(resolve(dest, ctx), Some((expected.to_string(), None)));
    }

    #[rstest]
    #[case::plain("../metrics/revenue.md#definition", Some("definition"))]
    #[case::percent_encoded("/a.md#Two%20Words", Some("Two Words"))]
    #[case::empty_fragment("/a.md#", None)]
    fn it_reads_the_fragment_as_a_heading(#[case] dest: &str, #[case] heading: Option<&str>) {
        let (_, actual) = resolve(dest, Some("tables")).unwrap();
        assert_eq!(actual.as_deref(), heading);
    }

    #[rstest]
    #[case::escapes_from_root("../a.md", None)]
    #[case::escapes_from_ctx("../../a.md", Some("tables"))]
    #[case::escapes_absolute("/../a.md", Some("tables"))]
    #[case::no_title("/tables/.md", None)]
    fn it_resolves_nothing_outside_the_wiki(#[case] dest: &str, #[case] ctx: Option<&str>) {
        assert_eq!(resolve(dest, ctx), None);
        assert!(MdLinkRef::new(dest, String::new(), 1).is_some());
    }

    #[rstest]
    #[case::https("https://example.com/a.md")]
    #[case::mailto("mailto:a@example.com")]
    #[case::custom_scheme("obsidian:vault/a.md")]
    #[case::protocol_relative("//example.com/a.md")]
    #[case::not_markdown("/tables/customers.html")]
    #[case::directory("/tables/")]
    #[case::query("/a.md?raw=1")]
    #[case::fragment_only("#heading")]
    #[case::uppercase_extension("/a.MD")]
    #[case::empty("")]
    fn it_rejects_what_is_not_a_scrap_path(#[case] dest: &str) {
        assert_eq!(MdLinkRef::new(dest, "text".to_string(), 1), None);
        assert_eq!(resolve(dest, None), None);
    }

    #[test]
    fn it_keeps_the_destination_as_written() {
        let link = MdLinkRef::new("../metrics/revenue.md#definition", "revenue".into(), 3).unwrap();
        assert_eq!(
            link.written(),
            "[revenue](../metrics/revenue.md#definition)"
        );
        assert_eq!(link.heading().as_deref(), Some("definition"));
        assert_eq!(
            link.resolve(&Some("tables".into())),
            Some(ScrapKey::from_path_str("metrics/revenue"))
        );
        assert_eq!(link.line, 3);
    }

    #[test]
    fn it_has_no_alias_without_link_text() {
        let link = MdLinkRef::new("/a.md", String::new(), 1).unwrap();
        assert_eq!(link.alias, None);
    }
}
