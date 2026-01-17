use crate::{error::ScrapsResult, usecase::read_scraps};
use scraps_libs::model::{scrap::Scrap, tags::Tags};
use std::path::PathBuf;

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct ListTagUsecase {
    scraps_dir_path: PathBuf,
}

impl ListTagUsecase {
    pub fn new(scraps_dir_path: &PathBuf) -> ListTagUsecase {
        ListTagUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }
    pub fn execute(&self) -> ScrapsResult<(Tags, BacklinksMap)> {
        let paths = read_scraps::to_scrap_paths(&self.scraps_dir_path)?;

        let scraps = paths
            .iter()
            .map(|path| read_scraps::to_scrap_by_path(&self.scraps_dir_path, path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        let tags = Tags::new(&scraps);
        let backlinks_map = BacklinksMap::new(&scraps);

        Ok((tags, backlinks_map))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use scraps_libs::model::tag::Tag;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_scrap("test1.md", b"#[[Tag1]] #[[Tag2]]")
            .add_scrap("test2.md", b"#[[Tag1]] #[[Tag3]]");

        let usecase = ListTagUsecase::new(&project.scraps_dir);

        let result = usecase.execute().unwrap();

        let (tags, backlinks_map) = result;

        // test tags
        let tag1: Tag = "Tag1".into();
        let tag2: Tag = "Tag2".into();
        let tag3: Tag = "Tag3".into();
        assert_eq!(tags.clone().into_iter().len(), 3);
        assert_eq!(
            tags.clone()
                .into_iter()
                .sorted_by_key(|t| t.title().to_string())
                .collect_vec(),
            vec![tag1.clone(), tag2.clone(), tag3.clone(),]
        );

        // test backlinks map
        let scrap1 = Scrap::new("test1", &None, "#[[Tag1]] #[[Tag2]]");
        let scrap2 = Scrap::new("test2", &None, "#[[Tag1]] #[[Tag3]]");
        assert_eq!(
            backlinks_map
                .get(&tag1.title().clone().into())
                .into_iter()
                .map(|s| s.title().clone())
                .sorted_by_key(|t| t.to_string())
                .collect_vec(),
            vec![scrap1.title().clone(), scrap2.title().clone()]
        );
        assert_eq!(
            backlinks_map
                .get(&tag2.title().clone().into())
                .into_iter()
                .map(|s| s.title().clone())
                .collect_vec(),
            vec![scrap1.title().clone()]
        );
        assert_eq!(
            backlinks_map
                .get(&tag3.title().clone().into())
                .into_iter()
                .map(|s| s.title().clone())
                .collect_vec(),
            vec![scrap2.title().clone()]
        );
    }
}
