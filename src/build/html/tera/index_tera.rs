use crate::build::model::{html::HtmlMetadata, sort::SortKey};
use once_cell::sync::Lazy;
use scraps_libs::error::{anyhow::Context, ScrapError, ScrapResult};
use tera::Tera;
use url::Url;

static INDEX_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("__builtins/base.html", include_str!("../builtins/base.html")),
        ("__builtins/index.html", include_str!("../builtins/index.html")),
        (
            "__builtins/macros.html",
            include_str!("../builtins/macros.html"),
        ),
    ])
    .unwrap();
    tera
});

pub fn init(
    base_url: &Url,
    metadata: &HtmlMetadata,
    sort_key: &SortKey,
    template_dir: &str,
) -> ScrapResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapError::PublicRender)?;
    tera.extend(&INDEX_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("base_url", &base_url);
    context.insert("title", &metadata.title());
    context.insert("description", &metadata.description());
    context.insert("favicon", &metadata.favicon());

    let sort_key_text = match sort_key {
        SortKey::CommittedDate => "commited date",
        SortKey::LinkedCount => "linked count",
    };
    context.insert("sort_key", sort_key_text);

    Ok((tera, context))
}
