use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::list_view_configs::ListViewConfigs;
use crate::usecase::build::model::scrap_detail::ScrapDetails;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use scraps_libs::model::tags::Tags;
use tracing::{span, Level};
use url::Url;

use crate::usecase::build::html::tera::index_tera;

use super::page_pointer::PagePointer;
use super::serde::index_scraps::IndexScrapsTera;
use super::serde::sort::SortKeyTera;
use super::serde::tags::TagsTera;

pub struct IndexRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl IndexRender {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path) -> ScrapsResult<IndexRender> {
        fs::create_dir_all(public_dir_path).context(BuildError::CreateDir)?;

        Ok(IndexRender {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
        })
    }

    pub fn run(
        &self,
        base_url: &Url,
        metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
    ) -> ScrapsResult<usize> {
        let scraps = &scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(scraps);
        let sorted_scraps = IndexScrapsTera::new_with_sort(
            scrap_details,
            &backlinks_map,
            &list_view_configs.sort_key,
        );
        let paginated = sorted_scraps
            .chunks(list_view_configs.paging.size_with(scraps))
            .into_par_iter()
            .enumerate();
        let last_page_num = paginated.len();
        let paginated_with_pointer = paginated.map(|(idx, paginated_scraps)| {
            let page_num = idx + 1;
            let pointer = PagePointer::new(page_num, last_page_num);
            (pointer, paginated_scraps)
        });
        let stags = &TagsTera::new(&Tags::new(scraps), &backlinks_map);

        paginated_with_pointer
            .clone()
            .try_for_each(|(pointer, paginated_scraps)| {
                let _span_generate_index = span!(Level::INFO, "generate_index").entered();
                let (tera, mut context) = index_tera::base(
                    base_url,
                    metadata,
                    self.static_dir_path.join("*.html").to_str().unwrap(),
                )?;
                context.insert(
                    "sort_key",
                    &SortKeyTera::from(list_view_configs.sort_key.clone()),
                );
                context.insert("build_search_index", &list_view_configs.build_search_index);
                context.insert("scraps", &paginated_scraps);
                context.insert("tags", stags);
                context.insert("prev", &pointer.prev);
                context.insert("next", &pointer.next);
                Self::render_paginated_html(self, &tera, &context, &pointer)?;
                ScrapsResult::Ok(())
            })?;

        Ok(paginated_with_pointer.len())
    }

    fn render_paginated_html(
        &self,
        tera: &tera::Tera,
        context: &tera::Context,
        pointer: &PagePointer,
    ) -> ScrapsResult<()> {
        let template_name = if tera.get_template_names().any(|t| t == "index.html") {
            "index.html"
        } else {
            "__builtins/index.html"
        };
        let file_path = &self.public_dir_path.join(pointer.current_file_name());
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to(template_name, context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use super::*;
    use crate::usecase::build::model::paging::Paging;
    use crate::usecase::build::model::scrap_detail::ScrapDetail;
    use crate::usecase::build::model::sort::SortKey;
    use scraps_libs::lang::LangCode;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::tests::FileResource;

    #[test]
    fn it_run() {
        // args
        let base_url = &Url::parse("http://localhost:1112/").unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let list_view_configs =
            ListViewConfigs::new(&true, &SortKey::CommittedDate, &Paging::By(2));

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_index_html_1");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_html_path = static_dir_path.join("index.html");
        let resource_template_html = FileResource::new(&template_html_path);
        let resource_template_html_byte =
        "{{ build_search_index }}{% for scrap in scraps %}<a href=\"./{{ scrap.title }}.html\">{{ scrap.title }}</a>{% endfor %}"
        .as_bytes();

        // scraps
        let scrap1 = Scrap::new("scrap1", &None, "# header1");
        let sc1 = ScrapDetail::new(&scrap1, &Some(1), base_url);
        let scrap2 = Scrap::new("scrap2", &None, "## header2");
        let sc2 = ScrapDetail::new(&scrap2, &Some(0), base_url);
        let scrap_details = ScrapDetails::new(&vec![sc1.to_owned(), sc2.to_owned()]);

        let index_html_path = public_dir_path.join("index.html");

        resource_template_html.run(resource_template_html_byte, || {
            let render = IndexRender::new(&static_dir_path, &public_dir_path).unwrap();
            render
                .run(base_url, &metadata, &list_view_configs, &scrap_details)
                .unwrap();

            let result1 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result1,
                "true<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );
        })
    }

    #[test]
    fn it_run_paging() {
        // args
        let base_url = &Url::parse("http://localhost:1112/").unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let list_view_configs =
            ListViewConfigs::new(&true, &SortKey::CommittedDate, &Paging::By(2));

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_index_html_2");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_html_path = static_dir_path.join("index.html");
        let resource_template_html = FileResource::new(&template_html_path);
        let resource_template_html_byte =
        "{{ build_search_index }}{% for scrap in scraps %}<a href=\"./{{ scrap.title }}.html\">{{ scrap.title }}</a>{% endfor %}"
        .as_bytes();

        // scraps
        let scrap1 = Scrap::new("scrap1", &None, "# header1");
        let sc1 = ScrapDetail::new(&scrap1, &Some(3), base_url);
        let scrap2 = Scrap::new("scrap2", &None, "## header2");
        let sc2 = ScrapDetail::new(&scrap2, &Some(2), base_url);
        let scrap3 = Scrap::new("scrap3", &None, "### header3");
        let sc3 = ScrapDetail::new(&scrap3, &Some(1), base_url);
        let scrap4 = Scrap::new("scrap4", &None, "#### header4");
        let sc4 = ScrapDetail::new(&scrap4, &Some(0), base_url);
        let scrap_details = ScrapDetails::new(&vec![
            sc1.to_owned(),
            sc2.to_owned(),
            sc3.to_owned(),
            sc4.to_owned(),
        ]);

        let index_html_path = public_dir_path.join("index.html");
        let page2_html_path = public_dir_path.join("2.html");

        resource_template_html.run(resource_template_html_byte, || {
            let render = IndexRender::new(&static_dir_path, &public_dir_path).unwrap();
            render
                .run(base_url, &metadata, &list_view_configs, &scrap_details)
                .unwrap();

            let result1 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result1,
                "true<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );

            let result2 = fs::read_to_string(page2_html_path).unwrap();
            assert_eq!(
                result2,
                "true<a href=\"./scrap3.html\">scrap3</a><a href=\"./scrap4.html\">scrap4</a>"
            );
        })
    }
}
