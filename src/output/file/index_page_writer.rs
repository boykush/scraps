use std::path::{Path, PathBuf};

use scraps_libs::model::{base_url::BaseUrl, content::Content};

use crate::error::ScrapsResult;
use crate::usecase::build::model::{
    backlinks_map::BacklinksMap, html::HtmlMetadata, list_view_configs::ListViewConfigs,
    scrap_detail::ScrapDetails,
};
use crate::usecase::build::port::IndexPageWriter;

use super::html::index_render::IndexRender;

pub struct FileIndexPageWriter {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    base_url: BaseUrl,
    html_metadata: HtmlMetadata,
}

impl FileIndexPageWriter {
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

impl IndexPageWriter for FileIndexPageWriter {
    fn write(
        &self,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize> {
        let render = IndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(
            &self.base_url,
            &self.html_metadata,
            list_view_configs,
            scrap_details,
            backlinks_map,
            readme_content,
        )
    }
}
