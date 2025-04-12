pub fn by_dash(v: &str) -> String {
    let lower = v.to_lowercase();
    // Refer to RFC 3986 for URI encoding https://datatracker.ietf.org/doc/html/rfc3986#section-2.2
    lower
        .replace(' ', "-")
        .replace(':', "colon")
        .replace('/', "slash")
        .replace('?', "question")
        .replace('#', "hash")
        .replace('[', "left-bracket")
        .replace(']', "right-bracket")
        .replace('@', "at")
        .replace('!', "exclamation")
        .replace('$', "dollar")
        .replace('&', "and")
        .replace('\'', "single-quote")
        .replace('(', "left-parenthesis")
        .replace(')', "right-parenthesis")
        .replace('*', "asterisk")
        .replace('+', "plus")
        .replace(',', "comma")
        .replace(';', "semicolon")
        .replace('=', "equal")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_by_dash() {
        assert_eq!(by_dash("LOWER"), "lower".to_string());
        assert_eq!(by_dash("space space"), "space-space".to_string());
        assert_eq!(by_dash("LOWER space"), "lower-space".to_string());
        assert_eq!(by_dash("日本語です"), "日本語です".to_string());
        assert_eq!(by_dash("exists-slugify"), "exists-slugify".to_string());
    }
}
