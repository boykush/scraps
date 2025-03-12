use std::fmt::Display;

use crate::slugify;

use super::{context::Ctx, title::Title};

#[derive(PartialEq, Clone, Debug, Eq, Hash, Ord, PartialOrd)]
pub struct Slug(String);

impl From<Title> for Slug {
    fn from(title: Title) -> Self {
        Slug(slugify::by_dash(&title.to_string()))
    }
}

impl From<Ctx> for Slug {
    fn from(ctx: Ctx) -> Self {
        Slug(slugify::by_dash(&ctx.to_string()))
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
