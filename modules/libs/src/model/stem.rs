use std::fmt::Display;

use crate::nfc::nfc;

/// A scrap's file name without `.md`: the last segment of its key.
#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Stem(String);

impl From<&str> for Stem {
    fn from(stem: &str) -> Self {
        Stem(nfc(stem).into_owned())
    }
}

impl Stem {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Stem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_new() {
        let stem: Stem = "scrap stem".into();
        assert_eq!(stem.0, "scrap stem".to_string());
    }

    #[test]
    fn it_is_the_same_stem_in_either_normalization_form() {
        let nfd: Stem = "\u{30ab}\u{3099}\u{30a4}\u{30c8}\u{3099}".into();
        let nfc: Stem = "\u{30ac}\u{30a4}\u{30c9}".into();
        assert_eq!(nfd, nfc);
        assert_eq!(nfd.to_string(), "\u{30ac}\u{30a4}\u{30c9}");
    }
}
