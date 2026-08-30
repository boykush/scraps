use std::fs;
use std::path::{Path, PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::list_view_configs::ListViewConfigs;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use crate::usecase::build::model::site_nav::SiteNav;
use crate::usecase::build::model::sort::SortKey;
use scraps_libs::model::{base_url::BaseUrl, content::Content};
use tera::Tera;
use tracing::{span, Level};

use crate::usecase::build::html::templates;

use super::page_pointer::PagePointer;
use super::serde::content::ContentTera;
use super::serde::index_scraps::IndexScrapsTera;

pub struct IndexRender {
    tera: Tera,
    output_dir_path: PathBuf,
}

impl IndexRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<IndexRender> {
        fs::create_dir_all(output_dir_path).context(BuildError::CreateDir)?;
        let tera = templates::index(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(IndexRender {
            tera,
            output_dir_path: output_dir_path.to_path_buf(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize> {
        let scraps = &scrap_details.to_scraps();
        let paging_size = list_view_configs.paging.size_with(scraps);
        let shared_context = templates::context(base_url, metadata);

        // Every sort view is always generated; a sort key is a URL, not a
        // config. The home is the updated view, README included.
        let updated_pages = {
            let _span = span!(Level::INFO, "generate_updated_view").entered();
            let sorted = IndexScrapsTera::new_with_sort(
                scrap_details,
                backlinks_map,
                &SortKey::CommittedDate,
            );
            self.render_view(
                &Self::view_context(&shared_context, "updated", site_nav, backlinks_map),
                &sorted,
                paging_size,
                readme_content,
                &self.output_dir_path,
            )?
        };

        let backlinks_pages = {
            let _span = span!(Level::INFO, "generate_backlinks_view").entered();
            let sorted =
                IndexScrapsTera::new_with_sort(scrap_details, backlinks_map, &SortKey::LinkedCount);
            self.render_view(
                &Self::view_context(&shared_context, "backlinks", site_nav, backlinks_map),
                &sorted,
                paging_size,
                &None,
                &self.output_dir_path.join("backlinks"),
            )?
        };

        Ok(updated_pages + backlinks_pages)
    }

    fn view_context(
        shared_context: &tera::Context,
        view: &str,
        site_nav: &SiteNav,
        backlinks_map: &BacklinksMap,
    ) -> tera::Context {
        let mut context = shared_context.clone();
        templates::insert_site_nav(&mut context, view, site_nav, backlinks_map);
        context
    }

    fn render_view(
        &self,
        view_context: &tera::Context,
        sorted_scraps: &IndexScrapsTera,
        paging_size: usize,
        readme_content: &Option<Content>,
        output_dir: &Path,
    ) -> ScrapsResult<usize> {
        let chunks = sorted_scraps.chunks(paging_size);
        let total_pages = chunks.len();

        if let Some(first_scraps) = chunks.first() {
            let (context, page_pointer) = Self::prepare_index_context(
                view_context,
                first_scraps,
                total_pages,
                readme_content,
            );
            self.render_html(&context, &page_pointer, output_dir)?;
        }

        chunks
            .iter()
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
                self.render_html(&context, &page_pointer, output_dir)?;
                ScrapsResult::Ok(())
            })?;

        Ok(total_pages)
    }

    fn prepare_index_context(
        base_context: &tera::Context,
        scraps: &IndexScrapsTera,
        total_pages: usize,
        readme_content: &Option<Content>,
    ) -> (tera::Context, PagePointer) {
        let pointer = PagePointer::new_index(total_pages);
        let mut context = base_context.clone();
        context.insert("scraps", &scraps);
        context.insert("next", &pointer.next);
        if let Some(readme) = readme_content {
            context.insert("readme_content", &ContentTera::from(readme.clone()));
        }
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
        output_dir: &Path,
    ) -> ScrapsResult<()> {
        let template_name = resolve_template(&self.tera, "index.html", "__builtins/index.html");
        let file_path = output_dir.join(pointer.current_file_name());
        render_to_file(&self.tera, template_name, context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
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
        let scrap_texts = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.md_text().to_string()))
            .collect();
        let sc1 = ScrapDetail::new(&scrap1, &Some(1), base_url, &scrap_texts);
        let sc2 = ScrapDetail::new(&scrap2, &Some(0), base_url, &scrap_texts);
        let scrap_details = ScrapDetails::new(&vec![sc1.to_owned(), sc2.to_owned()]);

        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true);

        let render = IndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(
                base_url,
                &metadata,
                &list_view_configs,
                &scrap_details,
                &backlinks_map,
                &site_nav,
                &None,
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
        let scrap_texts = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.md_text().to_string()))
            .collect();
        let sc1 = ScrapDetail::new(&scrap1, &Some(3), base_url, &scrap_texts);
        let sc2 = ScrapDetail::new(&scrap2, &Some(2), base_url, &scrap_texts);
        let sc3 = ScrapDetail::new(&scrap3, &Some(1), base_url, &scrap_texts);
        let sc4 = ScrapDetail::new(&scrap4, &Some(0), base_url, &scrap_texts);
        let scrap_details = ScrapDetails::new(&vec![
            sc1.to_owned(),
            sc2.to_owned(),
            sc3.to_owned(),
            sc4.to_owned(),
        ]);

        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true);

        let render = IndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        let readme_content: Option<Content> = None;
        render
            .run(
                base_url,
                &metadata,
                &list_view_configs,
                &scrap_details,
                &backlinks_map,
                &site_nav,
                &readme_content,
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
    /// fuse.js@7+ ships only an ES module from cdn.jsdelivr.net. Loading it
    /// with a classic `<script src=...>` tag makes the browser fail on the
    /// `export` statement, so `window.Fuse` never gets defined and the
    /// in-page search dies.
    ///
    /// We must (a) load fuse.js inside a `type="module"` script via `import`,
    /// and (b) avoid emitting a classic `<script src=".../fuse.js@...">` tag.
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
        let scrap_texts = [&scrap1]
            .iter()
            .map(|scrap| (scrap.self_key(), scrap.md_text().to_string()))
            .collect();
        let sc1 = ScrapDetail::new(&scrap1, &Some(0), base_url, &scrap_texts);
        let scrap_details = ScrapDetails::new(&vec![sc1]);
        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true);

        let render = IndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(
                base_url,
                &metadata,
                &list_view_configs,
                &scrap_details,
                &backlinks_map,
                &site_nav,
                &None,
            )
            .unwrap();

        let html = fs::read_to_string(project.output_path("index.html")).unwrap();

        assert!(
            !html.contains(r#"<script src="https://cdn.jsdelivr.net/npm/fuse.js@"#),
            "fuse.js@7+ is ESM-only: a classic <script src> tag fails to evaluate it. \
             Got HTML:\n{html}"
        );
        assert!(
            html.contains(r#"import Fuse from "https://cdn.jsdelivr.net/npm/fuse.js@"#),
            "fuse.js must be imported inside a `type=\"module\"` script. Got HTML:\n{html}"
        );
    }
}
