use crate::error::{BuildError, ScrapsResult, anyhow::Context};
use crate::usecase::build::html::cdn_versions::CDN_VERSIONS;
use crate::usecase::build::html::serde::scrap_type::{TypeTera, TypesTera};
use crate::usecase::build::html::serde::tags::TagsTera;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::scrap_types::ScrapType;
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::base_url::BaseUrl;
use std::sync::LazyLock;
use tera::Tera;

const BASE: (&str, &str) = ("__builtins/base.html", include_str!("builtins/base.html"));
const MACROS: (&str, &str) = (
    "__builtins/macros.html",
    include_str!("builtins/macros.html"),
);

static INDEX: LazyLock<Tera> =
    LazyLock::new(|| builtins(("__builtins/index.html", include_str!("builtins/index.html"))));
static ABOUT: LazyLock<Tera> =
    LazyLock::new(|| builtins(("__builtins/about.html", include_str!("builtins/about.html"))));
static SCRAP: LazyLock<Tera> =
    LazyLock::new(|| builtins(("__builtins/scrap.html", include_str!("builtins/scrap.html"))));
static TAG: LazyLock<Tera> =
    LazyLock::new(|| builtins(("__builtins/tag.html", include_str!("builtins/tag.html"))));
static SCRAPS_INDEX: LazyLock<Tera> = LazyLock::new(|| {
    builtins((
        "__builtins/scraps_index.html",
        include_str!("builtins/scraps_index.html"),
    ))
});
static TAGS_INDEX: LazyLock<Tera> = LazyLock::new(|| {
    builtins((
        "__builtins/tags_index.html",
        include_str!("builtins/tags_index.html"),
    ))
});
/// Every page kind inherits `base.html` and `macros.html`; `page` is the
/// template it renders through.
fn builtins(page: (&'static str, &'static str)) -> Tera {
    let mut tera = Tera::default();
    crate::service::tera_filters::register(&mut tera);
    tera.add_raw_templates(vec![BASE, MACROS, page])
        .expect("builtin templates are compiled into the binary");
    tera
}

/// Loading the glob re-parses every template, so callers rendering many pages
/// build this once and reuse it rather than call it per page.
fn with_user_templates(builtin: &Tera, template_dir: &str) -> ScrapsResult<Tera> {
    let mut tera = Tera::clone(builtin);
    tera.load_from_glob(template_dir)
        .context(BuildError::RenderHtml)?;
    Ok(tera)
}

pub fn index(template_dir: &str) -> ScrapsResult<Tera> {
    with_user_templates(&INDEX, template_dir)
}

pub fn about(template_dir: &str) -> ScrapsResult<Tera> {
    with_user_templates(&ABOUT, template_dir)
}

pub fn scrap(template_dir: &str) -> ScrapsResult<Tera> {
    with_user_templates(&SCRAP, template_dir)
}

pub fn tag(template_dir: &str) -> ScrapsResult<Tera> {
    with_user_templates(&TAG, template_dir)
}

pub fn scraps_index(template_dir: &str) -> ScrapsResult<Tera> {
    with_user_templates(&SCRAPS_INDEX, template_dir)
}

pub fn tags_index(template_dir: &str) -> ScrapsResult<Tera> {
    with_user_templates(&TAGS_INDEX, template_dir)
}

/// Context every HTML page starts from.
pub fn context(base_url: &BaseUrl, metadata: &HtmlMetadata) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("base_url", &base_url.as_url());
    context.insert("lang_code", &metadata.lang_code().to_string());
    context.insert("title", &metadata.title());
    context.insert("description", &metadata.description());
    context.insert("favicon", &metadata.favicon());
    context.insert("cdn", &CDN_VERSIONS);
    context.insert("scraps_version", env!("CARGO_PKG_VERSION"));

    context
}

/// The sidebar renders on every page, so every render inserts the same nav
/// payload; `view` names the sidebar entry to mark active ("" for none).
pub fn insert_site_nav(
    context: &mut tera::Context,
    view: &str,
    site_nav: &SiteNav,
    backlinks_map: &BacklinksMap,
) {
    context.insert("view", view);
    context.insert("scrap_count", &site_nav.scrap_count);
    context.insert("nav_tags", &TagsTera::new(&site_nav.tags, backlinks_map));
    context.insert("view_root", &base_url_of(context));
    context.insert("build_search_index", &site_nav.build_search_index);
    context.insert("timezone", &site_nav.timezone);
    context.insert("has_readme", &site_nav.has_readme);
}

fn base_url_of(context: &tera::Context) -> String {
    context
        .get("base_url")
        .and_then(tera::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// Lets a list view be narrowed to one type, the way a label narrows an issue
/// list: `view_path` is the view's path under the site root, and `active` the
/// type this page is narrowed to. A wiki without types gets no filter.
pub fn insert_type_filter(
    context: &mut tera::Context,
    view_path: &str,
    site_nav: &SiteNav,
    active: Option<&ScrapType>,
) {
    if site_nav.types.is_empty() {
        return;
    }
    context.insert("filter_types", &TypesTera::new(&site_nav.types));
    context.insert("view_path", view_path);
    if let Some(scrap_type) = active {
        // The sidebar's views keep the filter, so changing the sort order
        // does not drop the type the reader narrowed to.
        let view_root = format!(
            "{}{}",
            base_url_of(context),
            scrap_type.page_path().to_url_path()
        );
        context.insert("view_root", &view_root);
        context.insert("active_type", &TypeTera::from(scrap_type));
    }
}
