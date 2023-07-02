use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::build::model::scrap::Scrap;
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use crate::{build::css::render::CSSRender, libs::git::GitCommand};
use anyhow::{bail, Context};
use chrono_tz::Tz;
use url::Url;

use super::html::{index_render::IndexRender, scrap_render::ScrapRender};

pub struct BuildCommand {
    git_command: Box<dyn GitCommand>,
    scraps_dir_path: PathBuf,
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

#[derive(Clone)]
pub struct HtmlMetadata {
    pub title: String,
    pub description: Option<String>,
    pub favicon: Option<Url>,
}

impl BuildCommand {
    pub fn new(
        git_command: Box<dyn GitCommand>,
        scraps_dir_path: &PathBuf,
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
    ) -> BuildCommand {
        BuildCommand {
            git_command: git_command,
            scraps_dir_path: scraps_dir_path.to_owned(),
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }
    pub fn run(&self, timezone: &Tz, html_metadata: &HtmlMetadata) -> ScrapResult<()> {
        let read_dir = fs::read_dir(&self.scraps_dir_path).context(ScrapError::FileLoadError)?;

        let paths = read_dir
            .map(|entry_res| {
                let entry = entry_res?;
                self.to_path_by_dir_entry(&entry)
            })
            .collect::<ScrapResult<Vec<PathBuf>>>()?;

        let scraps = paths
            .iter()
            .map(|path| self.to_scrap_by_path(path))
            .collect::<ScrapResult<Vec<Scrap>>>()?;

        let index_render = IndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        index_render.run(
            timezone,
            &html_metadata.title,
            &html_metadata.description,
            &html_metadata.favicon,
            &scraps,
        )?;
        scraps.iter().map(|scrap| {
            let scrap_render = ScrapRender::new(&self.static_dir_path, &self.public_dir_path, &scraps)?;
            scrap_render.run(
                timezone,
                &html_metadata.title,
                &html_metadata.description,
                &html_metadata.favicon,
                scrap
            )
        }).collect::<ScrapResult<()>>()?;

        let css_render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        css_render.render_main()
    }

    fn to_path_by_dir_entry(&self, dir_entry: &DirEntry) -> ScrapResult<PathBuf> {
        if let Ok(file_type) = dir_entry.file_type() {
            if file_type.is_dir() {
                bail!(ScrapError::FileLoadError)
            }
        };
        Ok(dir_entry.path())
    }

    fn to_scrap_by_path(&self, path: &PathBuf) -> ScrapResult<Scrap> {
        let file_prefix = path
            .file_stem()
            .ok_or(ScrapError::FileLoadError)
            .map(|o| o.to_str())
            .and_then(|fp| fp.ok_or(ScrapError::FileLoadError.into()))?;
        let md_text = fs::read_to_string(&path).context(ScrapError::FileLoadError)?;
        let commited_ts = self.git_command.commited_ts(&path)?;

        Ok(Scrap::new(file_prefix, &md_text, &commited_ts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::{
        git::tests::GitCommandTest,
        resource::tests::{DirResource, FileResource},
    };

    #[test]
    fn it_run() {
        // args
        let timezone = chrono_tz::UTC;
        let html_metadata = &HtmlMetadata {
            title: "Scrap".to_string(),
            description: Some("Scrap Wiki".to_string()),
            favicon: Some(Url::parse("https://github.io/image.png").unwrap()),
        };
        let test_resource_path = PathBuf::from("tests/resource/build/cmd/it_run");
        let scraps_dir_path = test_resource_path.join("scraps");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");
        let git_command = GitCommandTest::new();

        // scrap1
        let md_path_1 = scraps_dir_path.join("test1.md");
        let html_path_1 = public_dir_path.join("test1.html");
        let resource_1 = FileResource::new(&md_path_1);
        let resource_bytes_1 = concat!("# header1\n", "## header2\n",).as_bytes();

        // scrap2
        let md_path_2 = scraps_dir_path.join("test2.md");
        let html_path_2 = public_dir_path.join("test2.html");
        let resource_2 = FileResource::new(&md_path_2);
        let resource_bytes_2 = concat!("[[test1]]\n").as_bytes();

        // static
        let resource_static_dir = DirResource::new(&static_dir_path);

        // public
        let html_path_3 = public_dir_path.join("index.html");
        let css_path = public_dir_path.join("main.css");

        resource_static_dir.run(|| {
            resource_1.run(resource_bytes_1, || {
                resource_2.run(resource_bytes_2, || {
                    let command = BuildCommand::new(
                        Box::new(git_command),
                        &scraps_dir_path,
                        &static_dir_path,
                        &public_dir_path,
                    );
                    let result1 = command.run(&timezone, &html_metadata);
                    assert!(result1.is_ok());

                    let result2 = fs::read_to_string(html_path_1);
                    assert!(result2.is_ok());

                    let result3 = fs::read_to_string(html_path_2);
                    assert!(result3.is_ok());

                    let result4 = fs::read_to_string(html_path_3);
                    assert!(result4.is_ok());

                    let result5 = fs::read_to_string(css_path);
                    assert!(result5.is_ok());
                })
            })
        })
    }
}
