use std::{
    marker::{Send, Sync},
    path::{Path, PathBuf},
};

use crate::error::{anyhow::Context, ScrapsResult};
use crate::{
    error::BuildError,
    usecase::{
        build::css::render::CSSRender,
        progress::{Progress, Stage},
    },
};
use chrono_tz::Tz;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use scraps_libs::{
    markdown,
    model::{base_url::BaseUrl, scrap::Scrap, tags::Tags},
};
use tracing::{span, Level};

use super::{
    html::{
        index_render::IndexRender, scrap_render::ScrapRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    model::{
        backlinks_map::BacklinksMap,
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_detail::{ScrapDetail, ScrapDetails},
    },
};
use crate::service::search::render::SearchIndexRender;

pub struct BuildUsecase {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl BuildUsecase {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path) -> BuildUsecase {
        BuildUsecase {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
        }
    }
    pub fn execute<PG: Progress>(
        &self,
        scraps_with_ts: &[(Scrap, Option<i64>)],
        readme_text: &Option<String>,
        progress: &PG,
        base_url: &BaseUrl,
        timezone: Tz,
        html_metadata: &HtmlMetadata,
        css_metadata: &CssMetadata,
        list_view_configs: &ListViewConfigs,
    ) -> ScrapsResult<usize> {
        progress.start_stage(&Stage::ReadScraps);
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
        progress.complete_stage(&Stage::ReadScraps, &scrap_details.len());

        // generate html
        progress.start_stage(&Stage::GenerateHtml);

        // generate html index
        let span_generate_html_indexes = span!(Level::INFO, "generate_html_indexes").entered();

        let index_render = IndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        let index_page_count = index_render.run(
            base_url,
            html_metadata,
            list_view_configs,
            &scrap_details,
            &backlinks_map,
            &readme_content,
        )?;
        span_generate_html_indexes.exit();

        // generate html scraps
        let span_generate_html_scraps = span!(Level::INFO, "generate_html_scraps").entered();
        scrap_details
            .to_vec()
            .into_par_iter()
            .try_for_each(|scrap_detail| {
                let _span_generate_html_scrap = span!(Level::INFO, "generate_html_scrap").entered();
                let scrap_render = ScrapRender::new(&self.static_dir_path, &self.public_dir_path)?;
                scrap_render.run(
                    base_url,
                    timezone,
                    html_metadata,
                    &scrap_detail,
                    &backlinks_map,
                )
            })?;
        span_generate_html_scraps.exit();

        // generate html tags index
        let span_generate_html_tags_index =
            span!(Level::INFO, "generate_html_tags_index").entered();
        let tags_index_render = TagsIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        tags_index_render.run(base_url, html_metadata, &scraps, &backlinks_map)?;
        span_generate_html_tags_index.exit();

        // generate html tags
        let span_generate_html_tags = span!(Level::INFO, "generate_html_tags").entered();
        let tags = Tags::new(&scraps);
        tags.iter().par_bridge().try_for_each(|tag| {
            let _span_render_tag = span!(Level::INFO, "generate_html_tag").entered();
            let tag_render = TagRender::new(&self.static_dir_path, &self.public_dir_path)?;
            tag_render.run(base_url, html_metadata, tag, &backlinks_map)
        })?;
        span_generate_html_tags.exit();

        progress.complete_stage(&Stage::GenerateHtml, &{
            index_page_count +
                scrap_details.len() +
                1 + // tags index
                tags.len()
        });

        // generate css
        progress.start_stage(&Stage::GenerateCss);
        let span_generate_css = span!(Level::INFO, "generate_css").entered();
        let css_render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        css_render.render_main(css_metadata)?;
        span_generate_css.exit();
        progress.complete_stage(&Stage::GenerateCss, &1);

        // generate search index json when build_search_index is true
        if list_view_configs.build_search_index {
            progress.start_stage(&Stage::GenerateJson);
            let _span_generate_json_search_index =
                span!(Level::INFO, "generate_json_search_index").entered();
            let search_index_render =
                SearchIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
            search_index_render.run(base_url, &scraps)?;
            progress.complete_stage(&Stage::GenerateJson, &1);
        }

        Ok(scraps.len())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::build::model::{color_scheme::ColorScheme, paging::Paging, sort::SortKey};
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
        let progress = ProgressTest::new();
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);
        let list_view_configs = ListViewConfigs::new(&true, &SortKey::LinkedCount, &Paging::Not);

        let usecase = BuildUsecase::new(&project.static_dir, &project.public_dir);
        let result1 = usecase
            .execute(
                &scraps_with_ts,
                &readme_text,
                &progress,
                &base_url,
                timezone,
                html_metadata,
                css_metadata,
                &list_view_configs,
            )
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
        let progress = ProgressTest::new();
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);
        let list_view_configs = ListViewConfigs::new(&false, &SortKey::LinkedCount, &Paging::Not);

        let usecase = BuildUsecase::new(&project.static_dir, &project.public_dir);
        let result1 = usecase
            .execute(
                &scraps_with_ts,
                &None,
                &progress,
                &base_url,
                timezone,
                html_metadata,
                css_metadata,
                &list_view_configs,
            )
            .unwrap();
        assert_eq!(result1, 2);

        // Verify search index JSON not generated
        let result2 = fs::read_to_string(project.public_path("search_index.json"));
        assert!(result2.is_err());
    }
}
