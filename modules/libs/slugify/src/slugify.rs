pub fn by_dash(v: &str) -> String {
    let lower = v.to_lowercase();
    lower.replace(' ', "-")
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
