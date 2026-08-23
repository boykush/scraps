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
    .expect("builtin templates are compiled into the binary");
    tera
});

/// Loading the glob re-parses every template, so build this once and reuse it.
pub fn tera(template_dir: &str) -> ScrapsResult<Tera> {
    let mut tera = Tera::clone(&SEARCH_INDEX_TERA);
    tera.load_from_glob(template_dir)
        .context(BuildError::RenderJson)?;

    Ok(tera)
}

pub fn context(base_url: &BaseUrl) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("base_url", base_url.as_url());

    context
}
