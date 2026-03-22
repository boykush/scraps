use crate::error::ScrapsResult;
use scraps_libs::model::{content::Content, scrap::Scrap, tag::Tag};

use super::model::{
    backlinks_map::BacklinksMap,
    list_view_configs::ListViewConfigs,
    scrap_detail::{ScrapDetail, ScrapDetails},
};

pub trait IndexPageWriter: Send + Sync {
    fn write_index_page(
        &self,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize>;
}

pub trait ScrapPageWriter: Send + Sync {
    fn write_scrap_page(
        &self,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()>;
}

pub trait TagPageWriter: Send + Sync {
    fn write_tags_index_page(
        &self,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()>;

    fn write_tag_page(&self, tag: &Tag, backlinks_map: &BacklinksMap) -> ScrapsResult<()>;
}

pub trait StyleWriter {
    fn write_style(&self) -> ScrapsResult<()>;
}

pub trait SearchIndexWriter {
    fn write_search_index(&self, scraps: &[Scrap]) -> ScrapsResult<()>;
}
