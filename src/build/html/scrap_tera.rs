use crate::{
    build::model::sort::SortKey,
    libs::error::{error::ScrapError, result::ScrapResult},
};
use anyhow::Context;
use chrono_tz::Tz;
use once_cell::sync::Lazy;
use tera::Tera;
use url::Url;

static SCRAP_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("__builtins/base.html", include_str!("builtins/base.html")),
        ("__builtins/index.html", include_str!("builtins/index.html")),
        (
            "__builtins/macros.html",
            include_str!("builtins/macros.html"),
        ),
        ("__builtins/scrap.html", include_str!("builtins/scrap.html")),
        ("__builtins/tag.html", include_str!("builtins/tag.html")),
    ])
    .unwrap();
    tera
});

pub fn init(
    timezone: &Tz,
    site_title: &str,
    site_description: &Option<String>,
    site_favicon: &Option<Url>,
    sort_key: &SortKey,
    template_dir: &str,
) -> ScrapResult<(Tera, tera::Context)> {
    let mut tera = Tera::new(template_dir).context(ScrapError::PublicRenderError)?;
    tera.extend(&SCRAP_TERA).unwrap();

    let mut context = tera::Context::new();
    context.insert("timezone", timezone);
    context.insert("title", site_title);
    context.insert("description", site_description);
    context.insert("favicon", site_favicon);

    let sort_key_text = match sort_key {
        SortKey::CommitedDate => "commited date",
        SortKey::LinkedCount => "linked count",
    };
    context.insert("sort_key", sort_key_text);

    Ok((tera, context))
}
