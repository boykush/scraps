use std::fs;
use std::path::Path;

use crate::error::BuildError;
use crate::error::{ScrapsResult, anyhow::Context};
use crate::service::tera_render::{RenderOutput, resolve_template, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::list_view_configs::ListViewConfigs;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use crate::usecase::build::model::site_nav::SiteNav;
use crate::usecase::build::model::sort::SortKey;
use rayon::prelude::*;
use scraps_libs::model::base_url::BaseUrl;
use tera::Tera;
use tracing::{Level, span};

use crate::usecase::build::html::templates;

use super::page_pointer::PagePointer;
use super::serde::index_scraps::IndexScrapsTera;

pub struct IndexRender {
    tera: Tera,
    output: RenderOutput,
}

impl IndexRender {
    pub fn new(static_dir_path: &Path, output: &RenderOutput) -> ScrapsResult<IndexRender> {
        if let RenderOutput::Dir(output_dir_path) = output {
            fs::create_dir_all(output_dir_path).context(BuildError::CreateDir)?;
        }
        let tera = templates::index(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(IndexRender {
            tera,
            output: output.clone(),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<usize> {
        let paging_size = list_view_configs.paging.size_with(scrap_details.len());
        let shared_context = templates::context(base_url, metadata);

        // Every sort view is always generated; a sort key is a URL, not a
        // config. The home is the updated view.
        let updated = {
            let _span = span!(Level::INFO, "generate_updated_view").entered();
            IndexScrapsTera::new_with_sort(scrap_details, backlinks_map, &SortKey::CommittedDate)
        };
        let backlinks = {
            let _span = span!(Level::INFO, "generate_backlinks_view").entered();
            IndexScrapsTera::new_with_sort(scrap_details, backlinks_map, &SortKey::LinkedCount)
        };

        // A type narrows a view the way a sort key orders it: as a URL. Each
        // type gets the same views again under its own directory.
        let scopes = std::iter::once(None).chain(site_nav.types.iter().map(Some));
        let mut pages = 0;
        for active in scopes {
            let output = match active {
                Some(scrap_type) => self.output.join(scrap_type.page_path().as_str()),
                None => self.output.clone(),
            };
            let of_scope = |sorted: &IndexScrapsTera| match active {
                Some(scrap_type) => sorted.of_type(&scrap_type.name),
                None => sorted.clone(),
            };
            let view_context = |view: &str, view_path: &str| {
                let mut context = shared_context.clone();
                templates::insert_site_nav(&mut context, view, site_nav, backlinks_map);
                templates::insert_type_filter(&mut context, view_path, site_nav, active);
                context
            };

            pages += self.render_view(
                &view_context("updated", ""),
                &of_scope(&updated),
                paging_size,
                &output,
            )?;
            pages += self.render_view(
                &view_context("backlinks", "backlinks/"),
                &of_scope(&backlinks),
                paging_size,
                &output.join("backlinks"),
            )?;
        }

        Ok(pages)
    }

    fn render_view(
        &self,
        view_context: &tera::Context,
        sorted_scraps: &IndexScrapsTera,
        paging_size: usize,
        output: &RenderOutput,
    ) -> ScrapsResult<usize> {
        let chunks = sorted_scraps.chunks(paging_size);
        let total_pages = chunks.len();

        if let Some(first_scraps) = chunks.first() {
            let (context, page_pointer) =
                Self::prepare_index_context(view_context, first_scraps, total_pages);
            self.render_html(&context, &page_pointer, output)?;
        }

        chunks
            .par_iter()
            .skip(1)
            .enumerate()
            .try_for_each(|(idx, page_scraps)| {
                let page_num = idx + 2;
                let (context, page_pointer) = Self::prepare_paginated_context(
                    view_context,
                    page_scraps,
                    page_num,
                    total_pages,
                );
                self.render_html(&context, &page_pointer, output)?;
                ScrapsResult::Ok(())
            })?;

        Ok(total_pages)
    }

    fn prepare_index_context(
        base_context: &tera::Context,
        scraps: &IndexScrapsTera,
        total_pages: usize,
    ) -> (tera::Context, PagePointer) {
        let pointer = PagePointer::new_index(total_pages);
        let mut context = base_context.clone();
        context.insert("scraps", &scraps);
        context.insert("next", &pointer.next);
        (context, pointer)
    }

    fn prepare_paginated_context(
        base_context: &tera::Context,
        scraps: &IndexScrapsTera,
        page_num: usize,
        total_pages: usize,
    ) -> (tera::Context, PagePointer) {
        let pointer = PagePointer::new_paginated(page_num, total_pages);
        let mut context = base_context.clone();
        context.insert("scraps", &scraps);
        context.insert("prev", &pointer.prev);
        context.insert("next", &pointer.next);
        (context, pointer)
    }

    fn render_html(
        &self,
        context: &tera::Context,
        pointer: &PagePointer,
        output: &RenderOutput,
    ) -> ScrapsResult<()> {
        let template_name = resolve_template(&self.tera, "index.html", "__builtins/index.html");
        output.render(
            &self.tera,
            template_name,
            context,
            pointer.current_file_name(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use crate::usecase::build::model::scrap_types::ScrapTypes;
    use rstest::rstest;
    use std::fs;
    use url::Url;

    use super::*;
    use crate::usecase::build::model::backlinks_map::BacklinksMap;
    use crate::usecase::build::model::paging::Paging;
    use crate::usecase::build::model::scrap_detail::ScrapDetail;
    use scraps_libs::lang::LangCode;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::model::tags::Tags;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Add static index.html template
        project.add_static_file(
            "index.html",
            b"{{ build_search_index }}{% for scrap in scraps %}<a href=\"./{{ scrap.title }}.html\">{{ scrap.title }}</a>{% endfor %}"
        );

        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let list_view_configs = ListViewConfigs::new(&true, &Paging::By(2));

        // scraps
        let scrap1 = Scrap::new("scrap1", &None, "# header1");
        let scrap2 = Scrap::new("scrap2", &None, "## header2");
        let scraps = [scrap1.clone(), scrap2.clone()];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let sc1 = ScrapDetail::new(&scrap1, &Some(1), base_url, &link_targets);
        let sc2 = ScrapDetail::new(&scrap2, &Some(0), base_url, &link_targets);
        let scrap_details = ScrapDetails::new(vec![sc1.to_owned(), sc2.to_owned()]);

        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        let render = IndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(
                base_url,
                &metadata,
                &list_view_configs,
                &scrap_details,
                &backlinks_map,
                &site_nav,
            )
            .unwrap();

        let result = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert_eq!(
            result,
            "true<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
        );

        // The backlinks view is always generated alongside the home. Both
        // scraps have zero backlinks, so the stable sort reversed flips the
        // input order.
        let backlinks_result =
            fs::read_to_string(project.output_path("backlinks/index.html")).unwrap();
        assert_eq!(
            backlinks_result,
            "true<a href=\"./scrap2.html\">scrap2</a><a href=\"./scrap1.html\">scrap1</a>"
        );
    }

    #[rstest]
    fn it_run_paging(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Add static index.html template
        project.add_static_file(
            "index.html",
            b"{{ build_search_index }}{% for scrap in scraps %}<a href=\"./{{ scrap.title }}.html\">{{ scrap.title }}</a>{% endfor %}"
        );

        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let list_view_configs = ListViewConfigs::new(&true, &Paging::By(2));

        // scraps
        let scrap1 = Scrap::new("scrap1", &None, "# header1");
        let scrap2 = Scrap::new("scrap2", &None, "## header2");
        let scrap3 = Scrap::new("scrap3", &None, "### header3");
        let scrap4 = Scrap::new("scrap4", &None, "#### header4");
        let scraps = [
            scrap1.clone(),
            scrap2.clone(),
            scrap3.clone(),
            scrap4.clone(),
        ];
        let link_targets = scraps_libs::html::LinkTargets::new(&scraps);
        let sc1 = ScrapDetail::new(&scrap1, &Some(3), base_url, &link_targets);
        let sc2 = ScrapDetail::new(&scrap2, &Some(2), base_url, &link_targets);
        let sc3 = ScrapDetail::new(&scrap3, &Some(1), base_url, &link_targets);
        let sc4 = ScrapDetail::new(&scrap4, &Some(0), base_url, &link_targets);
        let scrap_details = ScrapDetails::new(vec![
            sc1.to_owned(),
            sc2.to_owned(),
            sc3.to_owned(),
            sc4.to_owned(),
        ]);

        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        let render = IndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(
                base_url,
                &metadata,
                &list_view_configs,
                &scrap_details,
                &backlinks_map,
                &site_nav,
            )
            .unwrap();

        let index_result = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert_eq!(
            index_result,
            "true<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
        );

        let page2_result = fs::read_to_string(project.output_path("2.html")).unwrap();
        assert_eq!(
            page2_result,
            "true<a href=\"./scrap3.html\">scrap3</a><a href=\"./scrap4.html\">scrap4</a>"
        );
    }

    /// Regression test for the v1.0.0-rc.1 Fuse.js loading bug.
    ///
    /// fuse.js@7+ ships only an ES module. Loading it
    /// with a classic `<script src=...>` tag makes the browser fail on the
    /// `export` statement, so `window.Fuse` never gets defined and the
    /// in-page search dies.
    ///
    /// We must (a) load fuse.js inside a `type="module"` script via `import`,
    /// and (b) avoid emitting a classic `<script src=".../fuse.min.js">` tag.
    #[rstest]
    fn fusejs_loads_as_es_module_in_builtin_index(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        // Do NOT register a static index.html — we want the builtin template.

        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let list_view_configs = ListViewConfigs::new(&true, &Paging::By(10));

        let scrap1 = Scrap::new("scrap1", &None, "# header1");
        let link_targets = scraps_libs::html::LinkTargets::new([&scrap1]);
        let sc1 = ScrapDetail::new(&scrap1, &Some(0), base_url, &link_targets);
        let scrap_details = ScrapDetails::new(vec![sc1]);
        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        let render = IndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap();
        render
            .run(
                base_url,
                &metadata,
                &list_view_configs,
                &scrap_details,
                &backlinks_map,
                &site_nav,
            )
            .unwrap();

        let html = fs::read_to_string(project.output_path("index.html")).unwrap();

        assert!(
            !html.contains(r#"<script src="http://localhost:1112/js/fuse.min.js"#),
            "fuse.js@7+ is ESM-only: a classic <script src> tag fails to evaluate it. \
             Got HTML:\n{html}"
        );
        assert!(
            html.contains(r#"import Fuse from "http://localhost:1112/js/fuse.min.js"#),
            "fuse.js must be imported inside a `type=\"module\"` script. Got HTML:\n{html}"
        );
    }

    fn typed(title: &str, scrap_type: &str, body: &str) -> Scrap {
        Scrap::new(
            title,
            &None,
            &format!("---\ntype: {scrap_type}\n---\n{body}"),
        )
    }

    fn render_builtin(project: &TempScrapProject, scraps: &[Scrap], paging: Paging) -> usize {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);
        let link_targets = scraps_libs::html::LinkTargets::new(scraps);
        let scrap_details = ScrapDetails::new(
            scraps
                .iter()
                .map(|scrap| ScrapDetail::new(scrap, &None, base_url, &link_targets))
                .collect(),
        );
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(scraps),
            ScrapTypes::new(scraps),
            false,
            chrono_tz::UTC,
            false,
        );

        IndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap()
        .run(
            base_url,
            &metadata,
            &ListViewConfigs::new(&false, &paging),
            &scrap_details,
            &BacklinksMap::new(scraps),
            &site_nav,
        )
        .unwrap()
    }

    #[rstest]
    fn it_narrows_each_view_to_a_type(#[from(temp_scrap_project)] project: TempScrapProject) {
        let scraps = [
            typed("orders", "BigQuery Table", ""),
            typed("users", "BigQuery Table", "[[orders]]"),
            typed("setup", "How-to", "[[orders]]"),
            Scrap::new("untyped", &None, "[[orders]]"),
        ];

        let pages = render_builtin(&project, &scraps, Paging::Not);
        // updated and backlinks, unfiltered and once per type
        assert_eq!(pages, 6);

        let read = |path: &str| fs::read_to_string(project.output_path(path)).unwrap();

        // The unfiltered view offers every type, with "all" as the active one.
        let home = read("index.html");
        assert!(home.contains("<span class=\"option active\">all</span>"));
        assert!(home.contains(
            "href=\"http://localhost:1112/types/BigQuery%20Table/\">BigQuery Table<span class=\"count\">2</span>"
        ));
        assert!(home.contains("scraps/untyped.html"));

        let tables = read("types/BigQuery Table/index.html");
        assert!(tables.contains("scraps/orders.html"));
        assert!(tables.contains("scraps/users.html"));
        assert!(!tables.contains("scraps/setup.html"));
        assert!(!tables.contains("scraps/untyped.html"));
        assert!(tables.contains("<span class=\"stats\">2 of 4 scraps</span>"));
        assert!(tables.contains("<a class=\"option\" href=\"http://localhost:1112/\">all</a>"));
        assert!(tables.contains(
            "<span class=\"option active\">BigQuery Table<span class=\"count\">2</span></span>"
        ));
        // Changing the sort order keeps the filter.
        assert!(
            tables.contains("href=\"http://localhost:1112/types/BigQuery%20Table/backlinks/\"")
        );

        let most_linked = read("types/BigQuery Table/backlinks/index.html");
        assert!(most_linked.contains("<span class=\"item active\">backlinks</span>"));
        assert!(most_linked.contains("href=\"http://localhost:1112/backlinks/\">all</a>"));
        assert!(most_linked.find("scraps/orders.html") < most_linked.find("scraps/users.html"));
    }

    #[rstest]
    fn it_paginates_a_narrowed_view(#[from(temp_scrap_project)] project: TempScrapProject) {
        let scraps = [
            typed("a", "Blog", ""),
            typed("b", "Blog", ""),
            typed("c", "Blog", ""),
        ];

        render_builtin(&project, &scraps, Paging::By(2));

        assert!(project.output_path("types/Blog/index.html").exists());
        assert!(project.output_path("types/Blog/2.html").exists());
        assert!(!project.output_path("types/Blog/3.html").exists());
    }

    #[rstest]
    fn it_offers_no_filter_when_no_scrap_has_a_type(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let scraps = [Scrap::new("untyped", &None, "#[[Blog]]")];

        assert_eq!(render_builtin(&project, &scraps, Paging::Not), 2);

        let home = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert!(!home.contains("type-filter"));
        assert!(!project.output_path("types").exists());
    }
}
