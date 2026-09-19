/// Built in one pass into one buffer: the build slugifies every link and
/// tag on every page it renders.
pub fn by_dash(v: &str) -> String {
    let lower = v.trim().to_lowercase();
    let mut slug = String::with_capacity(lower.len());
    let mut in_word = false;
    for c in lower.chars() {
        if let Some(word) = reserved_word(c) {
            push_separator(&mut slug);
            slug.push_str(word);
            in_word = false;
        } else if c.is_whitespace() || c == '-' {
            in_word = false;
        } else {
            if !in_word {
                push_separator(&mut slug);
                in_word = true;
            }
            slug.push(c);
        }
    }
    slug
}

fn push_separator(slug: &mut String) {
    if !slug.is_empty() {
        slug.push('-');
    }
}

// Refer to RFC 3986 for URI encoding https://datatracker.ietf.org/doc/html/rfc3986#section-2.2
fn reserved_word(c: char) -> Option<&'static str> {
    let word = match c {
        ':' => "colon",
        '/' => "slash",
        '?' => "question",
        '#' => "hash",
        '[' => "left-bracket",
        ']' => "right-bracket",
        '@' => "at",
        '!' => "exclamation",
        '$' => "dollar",
        '&' => "and",
        '\'' => "single-quote",
        '(' => "left-parenthesis",
        ')' => "right-parenthesis",
        '*' => "asterisk",
        '+' => "plus",
        ',' => "comma",
        ';' => "semicolon",
        '=' => "equal",
        _ => return None,
    };
    Some(word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::lowercase("LOWER", "lower")]
    #[case::spaces("space space", "space-space")]
    #[case::lowercase_and_spaces("LOWER space", "lower-space")]
    #[case::japanese("日本語です", "日本語です")]
    #[case::existing_slugify("exists-slugify", "exists-slugify")]
    #[case::multiple_spaces("Multiple   Spaces   Here", "multiple-spaces-here")]
    #[case::leading_trailing_spaces("  Leading and Trailing  ", "leading-and-trailing")]
    #[case::special_chars_with_spaces("Hello, World!", "hello-comma-world-exclamation")]
    #[case::mixed_special_chars("Hello/Context@Test", "hello-slash-context-at-test")]
    #[case::empty("", "")]
    #[case::only_spaces("   ", "")]
    #[case::consecutive_special_chars(
        "Hello!!  @@World",
        "hello-exclamation-exclamation-at-at-world"
    )]
    #[case::dashes_and_spaces_collapse("a - b", "a-b")]
    #[case::edge_dashes("--a--", "a")]
    #[case::only_dashes("---", "")]
    #[case::dashed_replacement_words("[x]", "left-bracket-x-right-bracket")]
    #[case::ideographic_space("日本\u{3000}語", "日本-語")]
    fn test_by_dash(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(by_dash(input), expected);
    }
}
