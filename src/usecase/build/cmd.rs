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
use scraps_libs::{git::GitCommand, markdown, model::tags::Tags};
use tracing::{span, Level};
use url::Url;

use super::{
    html::{
        index_render::IndexRender, scrap_render::ScrapRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    json::render::SearchIndexRender,
    model::{
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_detail::{ScrapDetail, ScrapDetails},
    },
};

pub struct BuildCommand {
    scraps_dir_path: PathBuf,
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl BuildCommand {
    pub fn new(
        scraps_dir_path: &Path,
        static_dir_path: &Path,
        public_dir_path: &Path,
    ) -> BuildCommand {
        BuildCommand {
            scraps_dir_path: scraps_dir_path.to_path_buf(),
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
        }
    }
    pub fn run<GC: GitCommand + Send + Sync + Copy, PG: Progress>(
        &self,
        git_command: GC,
        progress: &PG,
        base_url: &Url,
        timezone: Tz,
        html_metadata: &HtmlMetadata,
        css_metadata: &CssMetadata,
        list_view_configs: &ListViewConfigs,
    ) -> ScrapsResult<usize> {
        progress.start_stage(&Stage::ReadScraps);
        let span_read_scraps = span!(Level::INFO, "read_scraps").entered();
        let paths = read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scrap_details = paths
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
        // Read README.md if it exists
        let readme_content = {
            let readme_path = self.scraps_dir_path.join("README.md");
            if readme_path.exists() {
                let md_text = fs::read_to_string(&readme_path).context(BuildError::ReadReadMe)?;
                Some(markdown::convert::to_content(&md_text, base_url))
            } else {
                None
            }
        };

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
                SearchIndexRender::new(&self.static_dir_path, &self.public_dir_path);
            search_index_render.run(base_url, &scraps)?;
            progress.complete_stage(&Stage::GenerateJson, &1);
        }

        Ok(scraps.len())
    }

    fn to_scrap_detail_by_path<GC: GitCommand + Send + Sync + Copy>(
        &self,
        git_command: GC,
        base_url: &Url,
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
    use std::path::Path;

    use crate::usecase::build::model::{color_scheme::ColorScheme, paging::Paging, sort::SortKey};
    use crate::usecase::progress::tests::ProgressTest;

    use super::*;
    use scraps_libs::{
        git::tests::GitCommandTest,
        lang::LangCode,
        tests::{DirResource, FileResource},
    };

    fn setup_command(test_resource_path: &Path) -> BuildCommand {
        let scraps_dir_path = test_resource_path.join("scraps");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");
        BuildCommand::new(&scraps_dir_path, &static_dir_path, &public_dir_path)
    }

    #[test]
    fn it_run() {
        // fields
        let test_resource_path = PathBuf::from("tests/resource/build/cmd/it_run");
        let command = setup_command(&test_resource_path);

        // run args
        let git_command = GitCommandTest::new();
        let progress = ProgressTest::new();
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);
        let list_view_configs = ListViewConfigs::new(&true, &SortKey::LinkedCount, &Paging::Not);

        // scrap1
        let md_path_1 = command.scraps_dir_path.join("test1.md");
        let html_path_1 = command.public_dir_path.join("scraps/test1.html");
        let resource_1 = FileResource::new(&md_path_1);
        let resource_bytes_1 = concat!("# header1\n", "## header2\n",).as_bytes();

        // scrap2
        let md_path_2 = command.scraps_dir_path.join("test2.md");
        let html_path_2 = command.public_dir_path.join("scraps/test2.html");
        let resource_2 = FileResource::new(&md_path_2);
        let resource_bytes_2 = concat!("[[test1]]\n").as_bytes();

        // excluded not markdown file
        let not_md_path = command.scraps_dir_path.join("test3.txt");
        let not_exists_path = command.public_dir_path.join("scraps/test3.html");
        let resource_3 = FileResource::new(&not_md_path);
        let resource_bytes_3 = concat!("# header1\n", "## header2\n",).as_bytes();

        // static
        let resource_static_dir = DirResource::new(&command.static_dir_path);

        // public
        let html_path_3 = command.public_dir_path.join("index.html");
        let css_path = command.public_dir_path.join("main.css");
        let search_index_json_path = command.public_dir_path.join("search_index.json");

        resource_static_dir.run(|| {
            resource_1.run(resource_bytes_1, || {
                resource_2.run(resource_bytes_2, || {
                    resource_3.run(resource_bytes_3, || {
                        let result1 = command
                            .run(
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

                        let result2 = fs::read_to_string(html_path_1).unwrap();
                        assert!(!result2.is_empty());

                        let result3 = fs::read_to_string(html_path_2).unwrap();
                        assert!(!result3.is_empty());

                        let result4 = fs::read_to_string(not_exists_path);
                        assert!(result4.is_err());

                        let result5 = fs::read_to_string(html_path_3).unwrap();
                        assert!(!result5.is_empty());

                        let result6 = fs::read_to_string(css_path).unwrap();
                        assert!(!result6.is_empty());

                        let result7 = fs::read_to_string(search_index_json_path).unwrap();
                        assert!(!result7.is_empty());
                    })
                })
            })
        })
    }

    #[test]
    fn it_run_when_build_search_index_is_false() {
        // fields
        let test_resource_path =
            PathBuf::from("tests/resource/build/cmd/it_run_when_build_search_index_is_false");
        let command = setup_command(&test_resource_path);

        // run args
        let git_command = GitCommandTest::new();
        let progress = ProgressTest::new();
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let css_metadata = &CssMetadata::new(&ColorScheme::OsSetting);
        let list_view_configs = ListViewConfigs::new(&false, &SortKey::LinkedCount, &Paging::Not);

        // scrap1
        let md_path_1 = command.scraps_dir_path.join("test1.md");
        let resource_1 = FileResource::new(&md_path_1);
        let resource_bytes_1 = concat!("# header1\n", "## header2\n",).as_bytes();

        // scrap2
        let md_path_2 = command.scraps_dir_path.join("test2.md");
        let resource_2 = FileResource::new(&md_path_2);
        let resource_bytes_2 = concat!("[[test1]]\n").as_bytes();

        // static
        let resource_static_dir = DirResource::new(&command.static_dir_path);

        // public
        let search_index_json_path = command.public_dir_path.join("search_index.json");

        resource_static_dir.run(|| {
            resource_1.run(resource_bytes_1, || {
                resource_2.run(resource_bytes_2, || {
                    let result1 = command
                        .run(
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

                    let result2 = fs::read_to_string(search_index_json_path);
                    assert!(result2.is_err());
                })
            })
        })
    }
}
