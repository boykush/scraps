use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::build::css::render::CSSRender;
use chrono_tz::Tz;
use scraps_libs::model::{scrap::Scrap, tags::Tags};
use scraps_libs::{
    error::{
        anyhow::{bail, Context},
        ScrapError, ScrapResult,
    },
    git::GitCommand,
};
use tracing::{span, Level};
use url::Url;

use super::{
    html::{
        index_render::IndexRender, scrap_render::ScrapRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    json::search_index_render::SearchIndexRender,
    model::{
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_with_commited_ts::{ScrapWithCommitedTs, ScrapsWithCommitedTs},
    },
};

pub struct BuildCommand<GC: GitCommand> {
    git_command: GC,
    scraps_dir_path: PathBuf,
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl<GC: GitCommand> BuildCommand<GC> {
    pub fn new(
        git_command: GC,
        scraps_dir_path: &PathBuf,
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
    ) -> BuildCommand<GC> {
        BuildCommand {
            git_command,
            scraps_dir_path: scraps_dir_path.to_owned(),
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }
    pub fn run(
        &self,
        base_url: &Url,
        timezone: Tz,
        html_metadata: &HtmlMetadata,
        css_metadata: &CssMetadata,
        list_view_configs: &ListViewConfigs,
    ) -> ScrapResult<usize> {
        let span_read_scraps = span!(Level::INFO, "read_scraps").entered();

        let read_dir = fs::read_dir(&self.scraps_dir_path).context(ScrapError::FileLoad)?;

        let paths = read_dir
            .map(|entry_res| {
                let entry = entry_res?;
                Self::to_path_by_dir_entry(&entry)
            })
            .collect::<ScrapResult<Vec<PathBuf>>>()?;

        let scraps_with_commited_ts = paths
            .iter()
            .map(|path| self.to_scrap_by_path(base_url, path))
            .collect::<ScrapResult<Vec<ScrapWithCommitedTs>>>()
            .map(|s| ScrapsWithCommitedTs::new(&s))?;
        let scraps = scraps_with_commited_ts.to_scraps();
        span_read_scraps.exit();

        // render index
        let span_render_index = span!(Level::INFO, "render_index").entered();
        let index_render = IndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        index_render.run(
            base_url,
            timezone,
            html_metadata,
            list_view_configs,
            &scraps_with_commited_ts,
        )?;
        span_render_index.exit();

        // render scraps
        let span_render_scraps = span!(Level::INFO, "render_scraps").entered();
        scraps_with_commited_ts
            .to_vec()
            .into_iter()
            .try_for_each(|scrap_with_commited_ts| {
                let _span_render_scrap = span!(Level::INFO, "render_scrap").entered();
                let scrap_render =
                    ScrapRender::new(&self.static_dir_path, &self.public_dir_path, &scraps)?;
                scrap_render.run(
                    base_url,
                    timezone,
                    html_metadata,
                    &scrap_with_commited_ts,
                    &list_view_configs.sort_key,
                )
            })?;
        span_render_scraps.exit();

        // render tags index
        let span_render_tags_index = span!(Level::INFO, "render_tags_index").entered();
        let tags_index_render = TagsIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        tags_index_render.run(
            base_url,
            timezone,
            html_metadata,
            &scraps,
            &list_view_configs.sort_key,
        )?;
        span_render_tags_index.exit();

        // render tag
        let span_render_tags = span!(Level::INFO, "render_tags").entered();
        let tags = Tags::new(&scraps);
        tags.values.iter().try_for_each(|tag| {
            let _span_render_tag = span!(Level::INFO, "render_tag").entered();
            let tag_render = TagRender::new(&self.static_dir_path, &self.public_dir_path, &scraps)?;
            tag_render.run(
                base_url,
                timezone,
                html_metadata,
                tag,
                &list_view_configs.sort_key,
            )
        })?;
        span_render_tags.exit();

        // render css
        let span_render_css = span!(Level::INFO, "render_css").entered();
        let css_render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        css_render.render_main(css_metadata)?;
        span_render_css.exit();

        // render search index json when build_search_index is true
        if list_view_configs.build_search_index {
            let _span_render_search_index = span!(Level::INFO, "render_search_index").entered();
            let search_index_render =
                SearchIndexRender::new(&self.static_dir_path, &self.public_dir_path);
            search_index_render.run(base_url, &scraps)?;
        }

        Ok(scraps.len())
    }

    fn to_path_by_dir_entry(dir_entry: &DirEntry) -> ScrapResult<PathBuf> {
        if let Ok(file_type) = dir_entry.file_type() {
            if file_type.is_dir() {
                bail!(ScrapError::FileLoad)
            }
        };
        Ok(dir_entry.path())
    }

    fn to_scrap_by_path(&self, base_url: &Url, path: &PathBuf) -> ScrapResult<ScrapWithCommitedTs> {
        let file_prefix = path
            .file_stem()
            .ok_or(ScrapError::FileLoad)
            .map(|o| o.to_str())
            .and_then(|fp| fp.ok_or(ScrapError::FileLoad))?;
        let md_text = fs::read_to_string(path).context(ScrapError::FileLoad)?;
        let scrap = Scrap::new(base_url, file_prefix, &md_text);
        let commited_ts = self.git_command.commited_ts(path)?;

        Ok(ScrapWithCommitedTs::new(&scrap, &commited_ts))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::build::{
        css,
        model::{color_scheme::ColorScheme, paging::Paging, sort::SortKey},
    };

    use super::*;
    use scraps_libs::{
        git::tests::GitCommandTest,
        tests::{DirResource, FileResource},
    };

    fn setup_command(test_resource_path: &Path) -> BuildCommand<GitCommandTest> {
        let git_command = GitCommandTest::new();
        let scraps_dir_path = test_resource_path.join("scraps");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");
        BuildCommand::new(
            git_command,
            &scraps_dir_path,
            &static_dir_path,
            &public_dir_path,
        )
    }

    #[test]
    fn it_run() {
        // fields
        let test_resource_path = PathBuf::from("tests/resource/build/cmd/it_run");
        let command = setup_command(&test_resource_path);

        // run args
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata::new(
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

        // static
        let resource_static_dir = DirResource::new(&command.static_dir_path);

        // public
        let html_path_3 = command.public_dir_path.join("index.html");
        let css_path = command.public_dir_path.join("main.css");
        let search_index_json_path = command.public_dir_path.join("search_index.json");

        resource_static_dir.run(|| {
            resource_1.run(resource_bytes_1, || {
                resource_2.run(resource_bytes_2, || {
                    let result1 = command.run(
                        &base_url,
                        timezone,
                        html_metadata,
                        css_metadata,
                        &list_view_configs,
                    );
                    assert!(result1.is_ok());

                    let result2 = fs::read_to_string(html_path_1);
                    assert!(result2.is_ok());

                    let result3 = fs::read_to_string(html_path_2);
                    assert!(result3.is_ok());

                    let result4 = fs::read_to_string(html_path_3);
                    assert!(result4.is_ok());

                    let result5 = fs::read_to_string(css_path);
                    assert!(result5.is_ok());

                    let result6 = fs::read_to_string(search_index_json_path);
                    assert!(result6.is_ok());
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
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata::new(
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
                    let result1 = command.run(
                        &base_url,
                        timezone,
                        html_metadata,
                        css_metadata,
                        &list_view_configs,
                    );
                    assert!(result1.is_ok());

                    let result2 = fs::read_to_string(search_index_json_path);
                    assert!(result2.is_err());
                })
            })
        })
    }
}
