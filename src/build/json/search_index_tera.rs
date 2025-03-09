use crate::error::{anyhow::Context, BuildError, ScrapsResult};
use once_cell::sync::Lazy;
use tera::Tera;
use url::Url;

static SEARCH_INDEX_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![(
        "__builtins/search_index.json",
        include_str!("builtins/search_index.json"),
    )])
    .unwrap();
    tera
});

pub fn base(base_url: &Url, template_dir: &str) -> ScrapsResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(BuildError::RenderJson)?;
    tera.extend(&SEARCH_INDEX_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("base_url", &base_url);

    Ok((tera, context))
}
