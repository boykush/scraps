use std::path::{Path, PathBuf};

use scraps_libs::model::{base_url::BaseUrl, scrap::Scrap, tag::Tag};

use crate::error::ScrapsResult;
use crate::usecase::build::model::{backlinks_map::BacklinksMap, html::HtmlMetadata};
use crate::usecase::build::port::TagPageWriter;

use super::html::{tag_render::TagRender, tags_index_render::TagsIndexRender};

pub struct FileTagPageWriter {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    base_url: BaseUrl,
    html_metadata: HtmlMetadata,
}

impl FileTagPageWriter {
    pub fn new(
        static_dir_path: &Path,
        public_dir_path: &Path,
        base_url: BaseUrl,
        html_metadata: HtmlMetadata,
    ) -> Self {
        Self {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
            base_url,
            html_metadata,
        }
    }
}

impl TagPageWriter for FileTagPageWriter {
    fn write_index(&self, scraps: &[Scrap], backlinks_map: &BacklinksMap) -> ScrapsResult<()> {
        let render = TagsIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(&self.base_url, &self.html_metadata, scraps, backlinks_map)
    }

    fn write_detail(&self, tag: &Tag, backlinks_map: &BacklinksMap) -> ScrapsResult<()> {
        let render = TagRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(&self.base_url, &self.html_metadata, tag, backlinks_map)
    }
}
