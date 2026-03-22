use std::path::{Path, PathBuf};

use scraps_libs::model::{base_url::BaseUrl, scrap::Scrap};

use crate::error::ScrapsResult;
use crate::usecase::build::port::SearchIndexWriter;

use super::json::render::SearchIndexRender;

pub struct FileSearchIndexWriter {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    base_url: BaseUrl,
}

impl FileSearchIndexWriter {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path, base_url: BaseUrl) -> Self {
        Self {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
            base_url,
        }
    }
}

impl SearchIndexWriter for FileSearchIndexWriter {
    fn write(&self, scraps: &[Scrap]) -> ScrapsResult<()> {
        let render = SearchIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(&self.base_url, scraps)
    }
}
