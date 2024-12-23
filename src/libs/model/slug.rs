use std::fmt::Display;

use scraps_libs::slugify;

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Slug(String);

impl Slug {
    pub(super) fn new(v: &str) -> Slug {
        let slug = slugify::by_dash(v);
        Slug(slug)
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
