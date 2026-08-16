use crate::error::{anyhow::Context, BuildError, ScrapsResult};
use once_cell::sync::Lazy;
use scraps_libs::model::base_url::BaseUrl;
use tera::Tera;

static SEARCH_INDEX_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    crate::service::tera_filters::register(&mut tera);
    tera.add_raw_templates(vec![(
        "__builtins/search_index.json",
        include_str!("builtins/search_index.json"),
    )])
    .unwrap();
    tera
});

pub fn base(base_url: &BaseUrl, template_dir: &str) -> ScrapsResult<(Tera, tera::Context)> {
    let mut tera = SEARCH_INDEX_TERA.clone();
    tera.load_from_glob(template_dir)
        .context(BuildError::RenderJson)?;

    let mut context = tera::Context::new();
    context.insert("base_url", base_url.as_url());

    Ok((tera, context))
}
