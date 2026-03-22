use crate::error::ScrapsResult;
use crate::usecase::progress::{Progress, Stage};
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use scraps_libs::{
    markdown,
    model::{base_url::BaseUrl, scrap::Scrap, tags::Tags},
};
use tracing::{span, Level};

use super::model::{
    backlinks_map::BacklinksMap,
    list_view_configs::ListViewConfigs,
    scrap_detail::{ScrapDetail, ScrapDetails},
};
use super::port::{
    IndexPageWriter, ScrapPageWriter, SearchIndexWriter, StyleWriter, TagPageWriter,
};

pub struct BuildUsecase<'a, IP, SP, TP, SW, SI, PG>
where
    IP: IndexPageWriter,
    SP: ScrapPageWriter,
    TP: TagPageWriter,
    SW: StyleWriter,
    SI: SearchIndexWriter,
    PG: Progress,
{
    index_page_writer: &'a IP,
    scrap_page_writer: &'a SP,
    tag_page_writer: &'a TP,
    style_writer: &'a SW,
    search_index_writer: &'a SI,
    progress: &'a PG,
}

impl<'a, IP, SP, TP, SW, SI, PG> BuildUsecase<'a, IP, SP, TP, SW, SI, PG>
where
    IP: IndexPageWriter,
    SP: ScrapPageWriter,
    TP: TagPageWriter,
    SW: StyleWriter,
    SI: SearchIndexWriter,
    PG: Progress,
{
    pub fn new(
        index_page_writer: &'a IP,
        scrap_page_writer: &'a SP,
        tag_page_writer: &'a TP,
        style_writer: &'a SW,
        search_index_writer: &'a SI,
        progress: &'a PG,
    ) -> Self {
        BuildUsecase {
            index_page_writer,
            scrap_page_writer,
            tag_page_writer,
            style_writer,
            search_index_writer,
            progress,
        }
    }

    pub fn execute(
        &self,
        scraps_with_ts: &[(Scrap, Option<i64>)],
        readme_text: &Option<String>,
        base_url: &BaseUrl,
        list_view_configs: &ListViewConfigs,
    ) -> ScrapsResult<usize> {
        self.progress.start_stage(&Stage::ReadScraps);
        let span_read_scraps = span!(Level::INFO, "read_scraps").entered();

        // Process README content
        let readme_content = readme_text
            .as_ref()
            .map(|text| markdown::convert::to_content(text, base_url));

        // Build ScrapDetails from pre-loaded data
        let scrap_details = scraps_with_ts
            .into_par_iter()
            .map(|(scrap, commited_ts)| ScrapDetail::new(scrap, commited_ts, base_url))
            .collect::<Vec<ScrapDetail>>();
        let scrap_details = ScrapDetails::new(&scrap_details);

        let scraps = scrap_details.to_scraps();
        let backlinks_map = BacklinksMap::new(&scraps);
        span_read_scraps.exit();
        self.progress
            .complete_stage(&Stage::ReadScraps, &scrap_details.len());

        // generate pages
        self.progress.start_stage(&Stage::GenerateHtml);

        // generate index page
        let span_generate_html_indexes = span!(Level::INFO, "generate_html_indexes").entered();
        let index_page_count = self.index_page_writer.write_index_page(
            list_view_configs,
            &scrap_details,
            &backlinks_map,
            &readme_content,
        )?;
        span_generate_html_indexes.exit();

        // generate scrap pages
        let span_generate_html_scraps = span!(Level::INFO, "generate_html_scraps").entered();
        scrap_details
            .to_vec()
            .into_par_iter()
            .try_for_each(|scrap_detail| {
                let _span_generate_html_scrap = span!(Level::INFO, "generate_html_scrap").entered();
                self.scrap_page_writer
                    .write_scrap_page(&scrap_detail, &backlinks_map)
            })?;
        span_generate_html_scraps.exit();

        // generate tags index page
        let span_generate_html_tags_index =
            span!(Level::INFO, "generate_html_tags_index").entered();
        self.tag_page_writer
            .write_tags_index_page(&scraps, &backlinks_map)?;
        span_generate_html_tags_index.exit();

        // generate tag pages
        let span_generate_html_tags = span!(Level::INFO, "generate_html_tags").entered();
        let tags = Tags::new(&scraps);
        tags.iter().par_bridge().try_for_each(|tag| {
            let _span_render_tag = span!(Level::INFO, "generate_html_tag").entered();
            self.tag_page_writer.write_tag_page(tag, &backlinks_map)
        })?;
        span_generate_html_tags.exit();

        self.progress.complete_stage(&Stage::GenerateHtml, &{
            index_page_count
                    + scrap_details.len()
                    + 1 // tags index
                    + tags.len()
        });

        // generate style
        self.progress.start_stage(&Stage::GenerateCss);
        let span_generate_css = span!(Level::INFO, "generate_css").entered();
        self.style_writer.write_style()?;
        span_generate_css.exit();
        self.progress.complete_stage(&Stage::GenerateCss, &1);

        // generate search index when build_search_index is true
        if list_view_configs.build_search_index {
            self.progress.start_stage(&Stage::GenerateJson);
            let _span_generate_json_search_index =
                span!(Level::INFO, "generate_json_search_index").entered();
            self.search_index_writer.write_search_index(&scraps)?;
            self.progress.complete_stage(&Stage::GenerateJson, &1);
        }

        Ok(scraps.len())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::output::file::build_output::{
        FileIndexPageWriter, FileScrapPageWriter, FileSearchIndexWriter, FileStyleWriter,
        FileTagPageWriter,
    };
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::build::model::{
        color_scheme::ColorScheme, css::CssMetadata, html::HtmlMetadata, paging::Paging,
        sort::SortKey,
    };
    use crate::usecase::progress::tests::ProgressTest;
    use rstest::rstest;

    use super::*;
    use scraps_libs::lang::LangCode;
    use url::Url;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Create scraps as (Scrap, Option<i64>) pairs
        let scraps_with_ts = vec![
            (
                Scrap::new("test1", &None, &concat!("# header1\n", "## header2\n")),
                Some(0i64),
            ),
            (Scrap::new("test2", &None, "[[test1]]\n"), Some(0i64)),
        ];

        let readme_text = Some("# README\n".to_string());

        // Run build
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let css_metadata = CssMetadata::new(&ColorScheme::OsSetting);
        let list_view_configs = ListViewConfigs::new(&true, &SortKey::LinkedCount, &Paging::Not);

        let index_page_writer = FileIndexPageWriter::new(
            &project.static_dir,
            &project.public_dir,
            base_url.clone(),
            html_metadata.clone(),
        );
        let scrap_page_writer = FileScrapPageWriter::new(
            &project.static_dir,
            &project.public_dir,
            base_url.clone(),
            timezone,
            html_metadata.clone(),
        );
        let tag_page_writer = FileTagPageWriter::new(
            &project.static_dir,
            &project.public_dir,
            base_url.clone(),
            html_metadata,
        );
        let style_writer =
            FileStyleWriter::new(&project.static_dir, &project.public_dir, css_metadata);
        let search_index_writer =
            FileSearchIndexWriter::new(&project.static_dir, &project.public_dir, base_url.clone());
        let progress = ProgressTest::new();
        let usecase = BuildUsecase::new(
            &index_page_writer,
            &scrap_page_writer,
            &tag_page_writer,
            &style_writer,
            &search_index_writer,
            &progress,
        );
        let result1 = usecase
            .execute(&scraps_with_ts, &readme_text, &base_url, &list_view_configs)
            .unwrap();
        assert_eq!(result1, 2);

        // Verify scrap1 HTML generated
        let result2 = fs::read_to_string(project.public_path("scraps/test1.html")).unwrap();
        assert!(!result2.is_empty());

        // Verify scrap2 HTML generated
        let result3 = fs::read_to_string(project.public_path("scraps/test2.html")).unwrap();
        assert!(!result3.is_empty());

        // Verify index.html generated
        let result6 = fs::read_to_string(project.public_path("index.html")).unwrap();
        assert!(!result6.is_empty());

        // Verify CSS generated
        let result7 = fs::read_to_string(project.public_path("main.css")).unwrap();
        assert!(!result7.is_empty());

        // Verify search index JSON generated
        let result8 = fs::read_to_string(project.public_path("search_index.json")).unwrap();
        assert!(!result8.is_empty());
    }

    #[rstest]
    fn it_run_when_build_search_index_is_false(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let scraps_with_ts = vec![
            (
                Scrap::new("test1", &None, &concat!("# header1\n", "## header2\n")),
                Some(0i64),
            ),
            (Scrap::new("test2", &None, "[[test1]]\n"), Some(0i64)),
        ];

        // Run build with search index disabled
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let css_metadata = CssMetadata::new(&ColorScheme::OsSetting);
        let list_view_configs = ListViewConfigs::new(&false, &SortKey::LinkedCount, &Paging::Not);

        let index_page_writer = FileIndexPageWriter::new(
            &project.static_dir,
            &project.public_dir,
            base_url.clone(),
            html_metadata.clone(),
        );
        let scrap_page_writer = FileScrapPageWriter::new(
            &project.static_dir,
            &project.public_dir,
            base_url.clone(),
            timezone,
            html_metadata.clone(),
        );
        let tag_page_writer = FileTagPageWriter::new(
            &project.static_dir,
            &project.public_dir,
            base_url.clone(),
            html_metadata,
        );
        let style_writer =
            FileStyleWriter::new(&project.static_dir, &project.public_dir, css_metadata);
        let search_index_writer =
            FileSearchIndexWriter::new(&project.static_dir, &project.public_dir, base_url.clone());
        let progress = ProgressTest::new();
        let usecase = BuildUsecase::new(
            &index_page_writer,
            &scrap_page_writer,
            &tag_page_writer,
            &style_writer,
            &search_index_writer,
            &progress,
        );
        let result1 = usecase
            .execute(&scraps_with_ts, &None, &base_url, &list_view_configs)
            .unwrap();
        assert_eq!(result1, 2);

        // Verify search index JSON not generated
        let result2 = fs::read_to_string(project.public_path("search_index.json"));
        assert!(result2.is_err());
    }
}
