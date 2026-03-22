use crate::error::ScrapsResult;
use scraps_libs::model::{content::Content, scrap::Scrap, tag::Tag};

use super::model::{
    backlinks_map::BacklinksMap,
    list_view_configs::ListViewConfigs,
    scrap_detail::{ScrapDetail, ScrapDetails},
};

pub trait IndexPageWriter: Send + Sync {
    fn write(
        &self,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize>;
}

pub trait ScrapPageWriter: Send + Sync {
    fn write(&self, scrap_detail: &ScrapDetail, backlinks_map: &BacklinksMap) -> ScrapsResult<()>;
}

pub trait TagPageWriter: Send + Sync {
    fn write_index(&self, scraps: &[Scrap], backlinks_map: &BacklinksMap) -> ScrapsResult<()>;

    fn write_detail(&self, tag: &Tag, backlinks_map: &BacklinksMap) -> ScrapsResult<()>;
}

pub trait StyleWriter {
    fn write(&self) -> ScrapsResult<()>;
}

pub trait SearchIndexWriter {
    fn write(&self, scraps: &[Scrap]) -> ScrapsResult<()>;
}
