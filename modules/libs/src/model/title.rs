use std::fmt::Display;

use crate::nfc::nfc;

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Title(String);

impl From<&str> for Title {
    fn from(title: &str) -> Self {
        Title(nfc(title).into_owned())
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_new() {
        let title: Title = "scrap title".into();
        assert_eq!(title.0, "scrap title".to_string());
    }

    #[test]
    fn it_is_the_same_title_in_either_normalization_form() {
        let nfd: Title = "\u{30ab}\u{3099}\u{30a4}\u{30c8}\u{3099}".into();
        let nfc: Title = "\u{30ac}\u{30a4}\u{30c9}".into();
        assert_eq!(nfd, nfc);
        assert_eq!(nfd.to_string(), "\u{30ac}\u{30a4}\u{30c9}");
    }
}
