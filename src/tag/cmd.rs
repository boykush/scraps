use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::{
    build::model::linked_scraps_map::LinkedScrapsMap,
    libs::{
        error::{ScrapError, ScrapResult},
        model::{scrap::Scrap, tags::Tags},
    },
};
use anyhow::{bail, Context};
use url::Url;

pub struct TagCommand {
    scraps_dir_path: PathBuf,
}

impl TagCommand {
    pub fn new(scraps_dir_path: &PathBuf) -> TagCommand {
        TagCommand {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }
    pub fn run(&self, base_url: &Url) -> ScrapResult<(Tags, LinkedScrapsMap)> {
        let read_dir = fs::read_dir(&self.scraps_dir_path).context(ScrapError::FileLoad)?;

        let paths = read_dir
            .map(|entry_res| {
                let entry = entry_res?;
                Self::to_path_by_dir_entry(&entry)
            })
            .collect::<ScrapResult<Vec<PathBuf>>>()?;

        let scraps = paths
            .iter()
            .map(|path| self.to_scrap_by_path(base_url, path))
            .collect::<ScrapResult<Vec<Scrap>>>()?;

        let tags = Tags::new(&scraps);
        let linked_scraps_map = LinkedScrapsMap::new(&scraps);

        Ok((tags, linked_scraps_map))
    }

    fn to_path_by_dir_entry(dir_entry: &DirEntry) -> ScrapResult<PathBuf> {
        if let Ok(file_type) = dir_entry.file_type() {
            if file_type.is_dir() {
                bail!(ScrapError::FileLoad)
            }
        };
        Ok(dir_entry.path())
    }

    fn to_scrap_by_path(&self, base_url: &Url, path: &PathBuf) -> ScrapResult<Scrap> {
        let file_prefix = path
            .file_stem()
            .ok_or(ScrapError::FileLoad)
            .map(|o| o.to_str())
            .and_then(|fp| fp.ok_or(ScrapError::FileLoad))?;
        let md_text = fs::read_to_string(path).context(ScrapError::FileLoad)?;
        let scrap = Scrap::new(base_url, file_prefix, &md_text);

        Ok(scrap)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::libs::resource::tests::FileResource;

    #[test]
    fn it_run() {
        // fields
        let test_resource_path = PathBuf::from("tests/resource/tag/cmd/it_run");
        let scraps_dir_path = test_resource_path.join("scraps");

        // run args
        let base_url = Url::parse("http://localhost:1112/").unwrap();

        // scrap1
        let md_path_1 = scraps_dir_path.join("test1.md");
        let resource_1 = FileResource::new(&md_path_1);
        let resource_bytes_1 = concat!("#[[Tag1]] #[[Tag2]",).as_bytes();

        // scrap2
        let md_path_2 = scraps_dir_path.join("test2.md");
        let resource_2 = FileResource::new(&md_path_2);
        let resource_bytes_2 = concat!("#[[Tag1]] #[[Tag3]]").as_bytes();

        resource_1.run(resource_bytes_1, || {
            resource_2.run(resource_bytes_2, || {
                let command = TagCommand::new(&scraps_dir_path);
                let result1 = command.run(&base_url);
                assert!(result1.is_ok());
            })
        })
    }
}
