use std::fmt::Display;

/// The context in which a scrap lives. A context represents a non-empty
/// hierarchical path (one or more segments). The "no context" / root case is
/// represented externally via `Option<Ctx> = None` — a `Ctx` value itself is
/// always non-empty.
#[derive(PartialEq, Clone, Debug, PartialOrd, Eq, Ord, Hash)]
pub struct Ctx {
    segments: Vec<String>,
}

impl Ctx {
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }
}

/// Parses a `/`-separated string. Empty segments (from leading, trailing, or
/// repeated `/`) are dropped. An entirely empty input yields a `Ctx` with no
/// segments — callers should typically use `Option<Ctx>::None` for that case;
/// this `From` is provided for ergonomic parsing of known-non-empty input.
impl From<&str> for Ctx {
    fn from(s: &str) -> Self {
        Ctx {
            segments: s
                .split('/')
                .filter(|seg| !seg.is_empty())
                .map(String::from)
                .collect(),
        }
    }
}

impl Display for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::single("Book", &["Book"][..])]
    #[case::two_levels("Book/Programming", &["Book", "Programming"][..])]
    #[case::three_levels("a/b/c", &["a", "b", "c"][..])]
    #[case::empty_input("", &[][..])]
    #[case::leading_slash("/a/b", &["a", "b"][..])]
    #[case::trailing_slash("a/b/", &["a", "b"][..])]
    #[case::double_slash_collapsed("a//b", &["a", "b"][..])]
    #[case::japanese("日本語/プログラミング", &["日本語", "プログラミング"][..])]
    #[case::emoji("🚀/notes", &["🚀", "notes"][..])]
    #[case::space_in_segment("Book/Test driven development", &["Book", "Test driven development"][..])]
    fn it_from_str_parsing(#[case] input: &str, #[case] expected: &[&str]) {
        let c: Ctx = input.into();
        let got: Vec<&str> = c.segments().iter().map(String::as_str).collect();
        assert_eq!(got, expected);
    }

    #[rstest]
    #[case::single("Book", "Book")]
    #[case::two_levels("a/b", "a/b")]
    #[case::three_levels("a/b/c", "a/b/c")]
    fn it_display_round_trip(#[case] input: &str, #[case] expected: &str) {
        let c: Ctx = input.into();
        assert_eq!(format!("{}", c), expected);
    }

    #[test]
    fn it_depth_matches_segment_count() {
        assert_eq!(Ctx::from("a").depth(), 1);
        assert_eq!(Ctx::from("a/b").depth(), 2);
        assert_eq!(Ctx::from("a/b/c").depth(), 3);
    }

    #[test]
    fn it_eq_and_hash_consider_full_path() {
        use std::collections::HashSet;
        let a: Ctx = "x/y".into();
        let b: Ctx = "x/y".into();
        let c: Ctx = "x/z".into();
        let d: Ctx = "x".into();

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
        let mut v: Vec<Ctx> = vec!["b".into(), "a/b".into(), "a".into(), "a/a".into()];
        v.sort();
        let displayed: Vec<String> = v.iter().map(|c| format!("{}", c)).collect();
        assert_eq!(displayed, vec!["a", "a/a", "a/b", "b"]);
    }
}
