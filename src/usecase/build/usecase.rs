use std::{
    marker::{Send, Sync},
    path::{Path, PathBuf},
};

use std::fs;

use crate::{
    error::BuildError,
    usecase::{
        build::css::render::CSSRender,
        progress::{Progress, Stage},
    },
};
use crate::{
    error::{anyhow::Context, ScrapsResult},
    usecase::read_scraps,
};
use chrono_tz::Tz;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use scraps_libs::{
    git::GitCommand,
    markdown,
    model::{base_url::BaseUrl, tags::Tags},
};
use tracing::{span, Level};

use super::{
    html::{
        index_render::IndexRender, scrap_render::ScrapRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    model::{
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_detail::{ScrapDetail, ScrapDetails},
    },
};
use crate::service::search::render::SearchIndexRender;

pub struct BuildUsecase {
    scraps_dir_path: PathBuf,
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl BuildUsecase {
    pub fn new(
        scraps_dir_path: &Path,
        static_dir_path: &Path,
        public_dir_path: &Path,
    ) -> BuildUsecase {
        BuildUsecase {
            scraps_dir_path: scraps_dir_path.to_path_buf(),
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
        }
    }
    pub fn execute<GC: GitCommand + Send + Sync + Copy, PG: Progress>(
        &self,
        git_command: GC,
        progress: &PG,
        base_url: &BaseUrl,
        timezone: Tz,
        html_metadata: &HtmlMetadata,
        css_metadata: &CssMetadata,
        list_view_configs: &ListViewConfigs,
    ) -> ScrapsResult<usize> {
        progress.start_stage(&Stage::ReadScraps);
        let span_read_scraps = span!(Level::INFO, "read_scraps").entered();
        let paths = read_scraps::to_scrap_paths(&self.scraps_dir_path)?;

        // Separate README.md and other scraps
        let readme_path = self.scraps_dir_path.join("README.md");
        let (readme_paths, scrap_paths): (Vec<_>, Vec<_>) =
            paths.into_iter().partition(|path| path == &readme_path);

        // Process README content
        let readme_path = readme_paths.first();
        let readme_content = match readme_path {
            Some(path) => {
                let readme_str = fs::read_to_string(path).context(BuildError::ReadREADMEFile)?;
                Some(markdown::convert::to_content(&readme_str, base_url))
            }
            None => None,
        };

        // Process other scraps in parallel
        let scrap_details = scrap_paths
            .into_par_iter()
            .map(|path| self.to_scrap_detail_by_path(git_command, base_url, &path))
            .collect::<ScrapsResult<Vec<ScrapDetail>>>()
            .map(|s| ScrapDetails::new(&s))?;

        let scraps = scrap_details.to_scraps();
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
                let scrap_render =
                    ScrapRender::new(&self.static_dir_path, &self.public_dir_path, &scraps)?;
                scrap_render.run(base_url, timezone, html_metadata, &scrap_detail)
            })?;
        span_generate_html_scraps.exit();

        // generate html tags index
        let span_generate_html_tags_index =
            span!(Level::INFO, "generate_html_tags_index").entered();
        let tags_index_render = TagsIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        tags_index_render.run(base_url, html_metadata, &scraps)?;
        span_generate_html_tags_index.exit();

        // generate html tags
        let span_generate_html_tags = span!(Level::INFO, "generate_html_tags").entered();
        let tags = Tags::new(&scraps);
        tags.clone().into_iter().par_bridge().try_for_each(|tag| {
            let _span_render_tag = span!(Level::INFO, "generate_html_tag").entered();
            let tag_render = TagRender::new(&self.static_dir_path, &self.public_dir_path, &scraps)?;
            tag_render.run(base_url, html_metadata, &tag)
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

    fn to_scrap_detail_by_path<GC: GitCommand + Send + Sync + Copy>(
        &self,
        git_command: GC,
        base_url: &BaseUrl,
        path: &PathBuf,
    ) -> ScrapsResult<ScrapDetail> {
        let span_convert_to_scrap = span!(Level::INFO, "convert_to_scrap").entered();
        let scrap = read_scraps::to_scrap_by_path(&self.scraps_dir_path, path)?;
        let commited_ts = git_command
            .commited_ts(path)
            .context(BuildError::GitCommitedTs)?;
        span_convert_to_scrap.exit();

        Ok(ScrapDetail::new(&scrap, &commited_ts, base_url))
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
    use scraps_libs::{git::tests::GitCommandTest, lang::LangCode};
    use url::Url;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Create scraps
        project
            .add_scrap(
                "test1.md",
                concat!("# header1\n", "## header2\n",).as_bytes(),
            )
            .add_scrap("test2.md", "[[test1]]\n".as_bytes());

        // Add non-markdown file (should be excluded)
        std::fs::write(
            project.scraps_dir.join("test3.txt"),
            concat!("# header1\n", "## header2\n",).as_bytes(),
        )
        .unwrap();

        // Add README.md (should not generate README.html)
        std::fs::write(
            project.scraps_dir.join("README.md"),
            "# README\n".as_bytes(),
        )
        .unwrap();

        // Run build
        let git_command = GitCommandTest::new();
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

        let usecase = BuildUsecase::new(
            &project.scraps_dir,
            &project.static_dir,
            &project.public_dir,
        );
        let result1 = usecase
            .execute(
                git_command,
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

        // Verify non-markdown file excluded
        let result4 = fs::read_to_string(project.public_path("scraps/test3.html"));
        assert!(result4.is_err());

        // Verify README.html not generated
        let result5 = fs::read_to_string(project.public_path("README.html"));
        assert!(result5.is_err());

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
        // Create scraps
        project
            .add_scrap(
                "test1.md",
                concat!("# header1\n", "## header2\n",).as_bytes(),
            )
            .add_scrap("test2.md", "[[test1]]\n".as_bytes());

        // Run build with search index disabled
        let git_command = GitCommandTest::new();
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

        let usecase = BuildUsecase::new(
            &project.scraps_dir,
            &project.static_dir,
            &project.public_dir,
        );
        let result1 = usecase
            .execute(
                git_command,
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
