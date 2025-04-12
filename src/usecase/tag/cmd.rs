use crate::{error::ScrapsResult, usecase::read_scraps};
use scraps_libs::model::{scrap::Scrap, tags::Tags};
use std::path::PathBuf;

use crate::usecase::build::model::backlinks_map::BacklinksMap;
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
    pub fn run(&self, base_url: &Url) -> ScrapsResult<(Tags, BacklinksMap)> {
        let paths = read_scraps::to_scrap_paths(&self.scraps_dir_path)?;

        let scraps = paths
            .iter()
            .map(|path| read_scraps::to_scrap_by_path(base_url, &self.scraps_dir_path, path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        let tags = Tags::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);

        Ok((tags, backlinks_map))
    }
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;

    use super::*;
    use scraps_libs::model::tag::Tag;
    use scraps_libs::tests::FileResource;

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
        let resource_bytes_1 = concat!("#[[Tag1]] #[[Tag2]]",).as_bytes();

        // scrap2
        let md_path_2 = scraps_dir_path.join("test2.md");
        let resource_2 = FileResource::new(&md_path_2);
        let resource_bytes_2 = concat!("#[[Tag1]] #[[Tag3]]").as_bytes();

        resource_1.run(resource_bytes_1, || {
            resource_2.run(resource_bytes_2, || {
                let command = TagCommand::new(&scraps_dir_path);

                let result = command.run(&base_url);
                println!("{:?}", result);
                assert!(result.is_ok());

                let (tags, backlinks_map) = result.unwrap();

                // test tags
                let tag1: Tag = "Tag1".into();
                let tag2: Tag = "Tag2".into();
                let tag3: Tag = "Tag3".into();
                assert_eq!(tags.clone().into_iter().len(), 3);
                assert_eq!(
                    tags.clone()
                        .into_iter()
                        .sorted_by_key(|t| t.title.to_string())
                        .collect_vec(),
                    vec![tag1.clone(), tag2.clone(), tag3.clone(),]
                );

                // test backlinks map
                let scrap1 = Scrap::new(&base_url, "test1", &None, "#[[Tag1]] #[[Tag2]]");
                let scrap2 = Scrap::new(&base_url, "test2", &None, "#[[Tag1]] #[[Tag3]]");
                assert_eq!(
                    backlinks_map
                        .get(&tag1.title.clone().into())
                        .into_iter()
                        .map(|s| s.title)
                        .sorted_by_key(|t| t.to_string())
                        .collect_vec(),
                    vec![scrap1.title.clone(), scrap2.title.clone()]
                );
                assert_eq!(
                    backlinks_map
                        .get(&tag2.title.clone().into())
                        .into_iter()
                        .map(|s| s.title)
                        .collect_vec(),
                    vec![scrap1.title.clone()]
                );
                assert_eq!(
                    backlinks_map
                        .get(&tag3.title.clone().into())
                        .into_iter()
                        .map(|s| s.title)
                        .collect_vec(),
                    vec![scrap2.title.clone()]
                );
            })
        })
    }
}
