use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::usecase::build::html::cdn_versions::CDN_VERSIONS;
use crate::usecase::build::model::html::HtmlMetadata;
use once_cell::sync::Lazy;
use scraps_libs::model::base_url::BaseUrl;
use tera::Tera;

static TAG_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    crate::service::tera_filters::register(&mut tera);
    tera.add_raw_templates(vec![
        (
            "__builtins/base.html",
            include_str!("../builtins/base.html"),
        ),
        (
            "__builtins/macros.html",
            include_str!("../builtins/macros.html"),
        ),
        ("__builtins/tag.html", include_str!("../builtins/tag.html")),
    ])
    .unwrap();
    tera
});

/// Loading the glob re-parses every template, so callers rendering many pages
/// should build this once and reuse it rather than call it per page.
pub fn tera(template_dir: &str) -> ScrapsResult<Tera> {
    let mut tera = TAG_TERA.clone();
    tera.load_from_glob(template_dir)
        .context(BuildError::RenderHtml)?;

    Ok(tera)
}

pub fn context(base_url: &BaseUrl, metadata: &HtmlMetadata) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("base_url", &base_url.as_url());
    context.insert("lang_code", &metadata.lang_code().to_string());
    context.insert("title", &metadata.title());
    context.insert("description", &metadata.description());
    context.insert("favicon", &metadata.favicon());
    context.insert("cdn", &CDN_VERSIONS);

    context
}
