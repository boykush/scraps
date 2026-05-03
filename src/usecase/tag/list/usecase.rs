use crate::error::ScrapsResult;
use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct ListTagUsecase;

impl ListTagUsecase {
    pub fn new() -> ListTagUsecase {
        ListTagUsecase
    }
    pub fn execute(&self, scraps: &[Scrap]) -> ScrapsResult<(Tags, BacklinksMap)> {
        let tags = Tags::new(scraps);
        let backlinks_map = BacklinksMap::new(scraps);

        Ok((tags, backlinks_map))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use scraps_libs::model::tag::Tag;

    #[test]
    fn it_run() {
        let scraps = vec![
            Scrap::new("test1", &None, "#[[Tag1]] #[[Tag2]]"),
            Scrap::new("test2", &None, "#[[Tag1]] #[[Tag3]]"),
        ];

        let usecase = ListTagUsecase::new();
        let (tags, backlinks_map) = usecase.execute(&scraps).unwrap();

        // test tags
        let tag1: Tag = "Tag1".into();
        let tag2: Tag = "Tag2".into();
        let tag3: Tag = "Tag3".into();
        assert_eq!(tags.clone().into_iter().len(), 3);
        assert_eq!(
            tags.clone()
                .into_iter()
                .sorted_by_key(|t| t.to_string())
                .collect_vec(),
            vec![tag1.clone(), tag2.clone(), tag3.clone(),]
        );

        // test backlinks map (tag-keyed, separate from scrap-link backlinks)
        let scrap1 = Scrap::new("test1", &None, "#[[Tag1]] #[[Tag2]]");
        let scrap2 = Scrap::new("test2", &None, "#[[Tag1]] #[[Tag3]]");
        assert_eq!(
            backlinks_map
                .get_tag(&tag1)
                .into_iter()
                .map(|s| s.title().clone())
                .sorted_by_key(|t| t.to_string())
                .collect_vec(),
            vec![scrap1.title().clone(), scrap2.title().clone()]
        );
        assert_eq!(
            backlinks_map
                .get_tag(&tag2)
                .into_iter()
                .map(|s| s.title().clone())
                .collect_vec(),
            vec![scrap1.title().clone()]
        );
        assert_eq!(
            backlinks_map
                .get_tag(&tag3)
                .into_iter()
                .map(|s| s.title().clone())
                .collect_vec(),
            vec![scrap2.title().clone()]
        );
    }
}
