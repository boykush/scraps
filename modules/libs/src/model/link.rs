use super::{context::Ctx, title::Title};

#[derive(PartialEq, Clone, Debug, PartialOrd, Eq, Ord, Hash)]
pub struct ScrapLink {
    pub title: Title,
    pub ctx: Option<Ctx>,
}

impl From<Title> for ScrapLink {
    fn from(title: Title) -> Self {
        ScrapLink { title, ctx: None }
    }
}

impl ScrapLink {
    pub fn new(title: &Title, ctx: &Option<Ctx>) -> ScrapLink {
        ScrapLink {
            title: title.clone(),
            ctx: ctx.clone(),
        }
    }

    pub fn with_ctx(title: &Title, ctx: &Ctx) -> ScrapLink {
        ScrapLink {
            title: title.clone(),
            ctx: Some(ctx.clone()),
        }
    }

    pub fn from_path_str(path: &str) -> ScrapLink {
        let parts = path.splitn(2, "/").collect::<Vec<&str>>();
        match parts[..] {
            [title] => ScrapLink::from(Title::from(title)),
            [ctx, title] => ScrapLink::with_ctx(&title.into(), &ctx.into()),
            _ => ScrapLink::from(Title::from("")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_from_path_str() {
        let only_title_path = ScrapLink::from_path_str("ctx/title");
        assert_eq!(only_title_path.title, "title".into());
        assert_eq!(only_title_path.ctx, Some("ctx".into()));

        let with_context_path = ScrapLink::from_path_str("title");
        assert_eq!(with_context_path.title, "title".into());
        assert_eq!(with_context_path.ctx, None);

        let nested_path = ScrapLink::from_path_str("ctx/title/extra");
        assert_eq!(nested_path.title, "title/extra".into());
        assert_eq!(nested_path.ctx, Some("ctx".into()));
    }
}
