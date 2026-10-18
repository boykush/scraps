use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::error::BuildError;
use crate::error::{ScrapsResult, anyhow::Context};
use crate::service::tera_render::{RenderOutput, resolve_template, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::list_view_configs::ListViewConfigs;
use crate::usecase::build::model::paging::Paging;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use crate::usecase::build::model::site_nav::SiteNav;
use rayon::prelude::*;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::page_path::PagePath;
use scraps_libs::model::tag::Tag;
use tera::Tera;
use tracing::{Level, span};

use crate::usecase::build::html::templates;

use super::serde::index_scraps::{IndexScraps, IndexScrapsTera};
use super::serde::scrap_type::TypesTera;

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

    /// Writes one list per scope: the whole wiki at the root and each tag
    /// under `tags/`. Sort order and type are query parameters the page
    /// applies to itself, so they add no pages. Returns the pages written.
    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<usize> {
        let rows = {
            let _span = span!(Level::INFO, "generate_index_rows").entered();
            IndexScraps::new(scrap_details, backlinks_map)
        };
        let paginate_by = match list_view_configs.paging {
            Paging::By(size) => size,
            Paging::Not => 0,
        };
        let shell = {
            let mut context = templates::context(base_url, metadata);
            context.insert("paginate_by", &paginate_by);
            context
        };

        self.render_scope(&shell, &rows.scope(|_| true), None, site_nav)?;
        site_nav.tags.iter().par_bridge().try_for_each(|tag| {
            let _span = span!(Level::INFO, "generate_tag_list").entered();
            let keys: HashSet<String> = backlinks_map
                .get_tag(tag)
                .iter()
                .map(|scrap| scrap.self_key().to_string())
                .collect();
            let list = rows.scope(|key| keys.contains(key));
            self.render_scope(&shell, &list, Some(tag), site_nav)
        })?;

        Ok(1 + site_nav.tags.len())
    }

    fn render_scope(
        &self,
        shell: &tera::Context,
        list: &IndexScrapsTera,
        tag: Option<&Tag>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut context = shell.clone();
        templates::insert_site_nav(&mut context, site_nav, tag, true);
        context.insert("scope_tag", &tag.map(Tag::to_string));
        context.insert("scraps", list);
        context.insert(
            "types",
            &TypesTera::counted_in(&site_nav.types, site_nav.scope_counts.of(tag)),
        );

        let output = match tag {
            Some(tag) => self.output.join(PagePath::of_tag(tag).as_str()),
            None => self.output.clone(),
        };
        let template_name = resolve_template(&self.tera, "index.html", "__builtins/index.html");
        output.render(&self.tera, template_name, &context, "index.html")
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rstest::rstest;
    use std::fs;
    use url::Url;

    use super::*;
    use crate::usecase::build::model::scrap_detail::ScrapDetail;
    use scraps_libs::lang::LangCode;
    use scraps_libs::model::scrap::Scrap;

    fn render(project: &TempScrapProject, scraps: &[Scrap], ts: &[Option<i64>]) -> usize {
        let base_url = &BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(&LangCode::default(), "Scrap", &None, &None);
        let link_targets = scraps_libs::html::LinkTargets::new(scraps);
        let scrap_details = ScrapDetails::new(
            scraps
                .iter()
                .zip(ts.iter().chain(std::iter::repeat(&None)))
                .map(|(scrap, ts)| ScrapDetail::new(scrap, ts, base_url, &link_targets))
                .collect(),
        );
        let backlinks_map = BacklinksMap::new(scraps);
        let site_nav = SiteNav::new(scraps, &backlinks_map, true, chrono_tz::UTC, false);

        IndexRender::new(
            &project.static_dir,
            &RenderOutput::Dir(project.output_dir.clone()),
        )
        .unwrap()
        .run(
            base_url,
            &metadata,
            &ListViewConfigs::new(&true, &Paging::By(2)),
            &scrap_details,
            &backlinks_map,
            &site_nav,
        )
        .unwrap()
    }

    fn typed(title: &str, scrap_type: &str, body: &str) -> Scrap {
        Scrap::new(
            title,
            &None,
            &format!("---\ntype: {scrap_type}\n---\n{body}"),
        )
    }

    #[rstest]
    fn it_lists_every_scrap_in_updated_order(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_static_file(
            "index.html",
            b"{{ paginate_by }}{% for row in scraps.rows %} {{ row.title }}:{{ row.backlinks_rank }}{% endfor %}",
        );
        let scraps = [
            Scrap::new("old", &None, ""),
            Scrap::new("new", &None, "[[old]]"),
            Scrap::new("mid", &None, ""),
        ];

        let pages = render(&project, &scraps, &[Some(0), Some(2), Some(1)]);

        assert_eq!(pages, 1);
        // Every row is on the one page, whatever `paginate_by` says.
        let home = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert_eq!(home, "2 new:1 mid:2 old:0");
        assert!(!project.output_path("2.html").exists());
        assert!(!project.output_path("backlinks").exists());
        assert!(!project.output_path("all-scraps").exists());
    }

    #[rstest]
    fn it_writes_a_list_for_each_tag(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_static_file(
            "index.html",
            b"{{ scope_tag }}|{% for row in scraps.rows %}{{ row.title }} {% endfor %}|{% for t in types %}{{ t.name }}={{ t.scraps_count }} {% endfor %}",
        );
        let scraps = [
            typed("a", "Book", "#[[ai/ml]]"),
            typed("b", "Blog", "#[[ai]]"),
            typed("c", "Book", "#[[other]]"),
        ];

        assert_eq!(render(&project, &scraps, &[]), 4);

        let read = |path: &str| fs::read_to_string(project.output_path(path)).unwrap();
        // A parent tag lists what its children carry; type counts are the scope's.
        assert_eq!(read("tags/ai/index.html"), "ai|b a |Book=1 Blog=1 ");
        assert_eq!(read("tags/ai/ml/index.html"), "ai/ml|a |Book=1 Blog=0 ");
        assert_eq!(read("tags/other/index.html"), "other|c |Book=1 Blog=0 ");
        assert!(!project.output_path("types").exists());
        assert!(!project.output_path("tags/ai.html").exists());
    }

    #[rstest]
    fn it_marks_each_row_for_the_page_to_filter_and_sort(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let scraps = [
            typed("Rust book", "Book", "#[[Programming]]"),
            Scrap::new("Ownership", &None, "#[[Programming]] [[Rust book]]"),
        ];

        render(&project, &scraps, &[Some(1), Some(2)]);

        let html = fs::read_to_string(project.output_path("tags/Programming/index.html")).unwrap();
        assert!(html.contains(r#"data-type="Book""#), "{html}");
        assert!(html.contains(r#"data-rank-backlinks="0""#), "{html}");
        // The scope the page lists is the sidebar entry marked current.
        assert!(
            html.contains(r#"aria-current="page""#) && html.contains("Programming"),
            "{html}"
        );
        // A type a scope holds none of is offered but cannot be picked.
        let home = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert!(home.contains(r#"data-type="Book""#), "{home}");
    }

    #[rstest]
    fn it_offers_no_type_facet_when_no_scrap_has_a_type(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let scraps = [Scrap::new("untyped", &None, "#[[Blog]]")];

        render(&project, &scraps, &[]);

        let home = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert!(home.contains("facet-sort"), "{home}");
        assert!(!home.contains("facet-type"), "{home}");
    }

    #[rstest]
    fn fusejs_loads_as_es_module_in_builtin_index(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        // Regression for v1.0.0-rc.1: fuse.js@7+ is ESM-only, so a classic
        // `<script src>` leaves `window.Fuse` undefined and search dies.
        render(&project, &[Scrap::new("scrap1", &None, "# header1")], &[]);

        let html = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert!(
            !html.contains(r#"<script src="http://localhost:1112/js/fuse.min.js"#),
            "Got HTML:\n{html}"
        );
        assert!(
            html.contains(r#"import Fuse from "http://localhost:1112/js/fuse.min.js"#),
            "Got HTML:\n{html}"
        );
    }
}
