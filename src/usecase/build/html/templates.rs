use crate::error::{BuildError, ScrapsResult, anyhow::Context};
use crate::usecase::build::css::serde::color_scheme::ColorSchemeTera;
use crate::usecase::build::html::serde::nav_scopes::NavScopesTera;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::{base_url::BaseUrl, tag::Tag};
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
    context.insert(
        "color_scheme",
        &ColorSchemeTera::new(metadata.color_scheme()),
    );
    context.insert("scraps_version", env!("CARGO_PKG_VERSION"));

    context
}

/// The sidebar renders on every page, so every render inserts the same nav
/// payload; `active` is the tag a list page is scoped to, `None` on the
/// whole wiki's list and on pages that list nothing.
pub fn insert_site_nav(
    context: &mut tera::Context,
    site_nav: &SiteNav,
    active: Option<&Tag>,
    is_list: bool,
) {
    context.insert("scrap_count", &site_nav.scrap_count);
    context.insert("nav_scopes", &NavScopesTera::new(site_nav, active, is_list));
    context.insert("build_search_index", &site_nav.build_search_index);
    context.insert("timezone", &site_nav.timezone);
    context.insert("has_readme", &site_nav.has_readme);
}
