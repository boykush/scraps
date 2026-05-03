use std::fmt::Display;

/// A tag is a hierarchical label attached to a scrap via the `#[[a/b/c]]`
/// syntax. A tag is non-empty by convention; constructing an empty tag via
/// `From<&str>` is possible but should be avoided by callers.
#[derive(Eq, Hash, PartialEq, Debug, Clone, PartialOrd, Ord)]
pub struct Tag {
    segments: Vec<String>,
}

impl Tag {
    /// All segments of the tag, e.g. `["a", "b", "c"]` for `#[[a/b/c]]`.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// The leaf segment of the tag — the most specific label.
    pub fn name(&self) -> &str {
        self.segments.last().map(String::as_str).unwrap_or_default()
    }

    /// All proper ancestor tags, from the root toward (but not including) self.
    /// Used for Logseq-style auto-aggregation: a scrap tagged `#[[a/b/c]]`
    /// is also implicitly tagged with `a/b` and `a`.
    pub fn ancestors(&self) -> Vec<Tag> {
        (1..self.segments.len())
            .map(|n| Tag {
                segments: self.segments[..n].to_vec(),
            })
            .collect()
    }
}

/// Parse a `/`-separated string into a hierarchical tag. Empty segments
/// (from leading, trailing, or repeated slashes) are dropped.
impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Tag {
            segments: s
                .split('/')
                .filter(|seg| !seg.is_empty())
                .map(String::from)
                .collect(),
        }
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Tag::from(s.as_str())
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::single("foo", &["foo"][..])]
    #[case::two_levels("a/b", &["a", "b"][..])]
    #[case::three_levels("a/b/c", &["a", "b", "c"][..])]
    #[case::leading_slash("/a/b", &["a", "b"][..])]
    #[case::trailing_slash("a/b/", &["a", "b"][..])]
    #[case::double_slash_collapsed("a//b", &["a", "b"][..])]
    #[case::japanese("日本語/プログラミング", &["日本語", "プログラミング"][..])]
    #[case::emoji("🚀/notes", &["🚀", "notes"][..])]
    #[case::space_in_segment("Domain Driven Design", &["Domain Driven Design"][..])]
    fn it_from_str_parsing(#[case] input: &str, #[case] expected: &[&str]) {
        let t: Tag = input.into();
        let got: Vec<&str> = t.segments().iter().map(String::as_str).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn it_depth_matches_segment_count() {
        assert_eq!(Tag::from("a").depth(), 1);
        assert_eq!(Tag::from("a/b").depth(), 2);
        assert_eq!(Tag::from("a/b/c").depth(), 3);
    }

    #[test]
    fn it_name_returns_leaf_segment() {
        assert_eq!(Tag::from("foo").name(), "foo");
        assert_eq!(Tag::from("a/b/c").name(), "c");
        assert_eq!(Tag::from("Programming/Rust").name(), "Rust");
    }

    #[rstest]
    #[case::flat("foo", vec![])]
    #[case::two_levels("a/b", vec!["a"])]
    #[case::three_levels("a/b/c", vec!["a", "a/b"])]
    #[case::four_levels("a/b/c/d", vec!["a", "a/b", "a/b/c"])]
    fn it_ancestors_are_proper_prefixes(#[case] input: &str, #[case] expected: Vec<&str>) {
        let tag = Tag::from(input);
        let ancestors: Vec<String> = tag.ancestors().iter().map(|t| format!("{}", t)).collect();
        let expected_strs: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(ancestors, expected_strs);
    }

    #[rstest]
    #[case::single("foo", "foo")]
    #[case::two_levels("a/b", "a/b")]
    #[case::three_levels("a/b/c", "a/b/c")]
    fn it_display_round_trip(#[case] input: &str, #[case] expected: &str) {
        let t: Tag = input.into();
        assert_eq!(format!("{}", t), expected);
    }

    #[test]
    fn it_eq_and_hash_consider_full_path() {
        use std::collections::HashSet;
        let a: Tag = "x/y".into();
        let b: Tag = "x/y".into();
        let c: Tag = "x/z".into();
        let d: Tag = "x".into();

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);

        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }

    #[test]
    fn it_ord_is_segment_lexicographic() {
        let mut v: Vec<Tag> = vec!["b".into(), "a/b".into(), "a".into(), "a/a".into()];
        v.sort();
        let displayed: Vec<String> = v.iter().map(|t| format!("{}", t)).collect();
        assert_eq!(displayed, vec!["a", "a/a", "a/b", "b"]);
    }

    #[test]
    fn it_from_string_owned() {
        let owned = String::from("a/b/c");
        let t: Tag = owned.into();
        assert_eq!(format!("{}", t), "a/b/c");
    }
}
