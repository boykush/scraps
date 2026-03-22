use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use scraps_libs::model::base_url::BaseUrl;

use crate::error::ScrapsResult;
use crate::usecase::build::model::{
    backlinks_map::BacklinksMap, html::HtmlMetadata, scrap_detail::ScrapDetail,
};
use crate::usecase::build::port::ScrapPageWriter;

use super::html::scrap_render::ScrapRender;

pub struct FileScrapPageWriter {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    base_url: BaseUrl,
    timezone: Tz,
    html_metadata: HtmlMetadata,
}

impl FileScrapPageWriter {
    pub fn new(
        static_dir_path: &Path,
        public_dir_path: &Path,
        base_url: BaseUrl,
        timezone: Tz,
        html_metadata: HtmlMetadata,
    ) -> Self {
        Self {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
            base_url,
            timezone,
            html_metadata,
        }
    }
}

impl ScrapPageWriter for FileScrapPageWriter {
    fn write(&self, scrap_detail: &ScrapDetail, backlinks_map: &BacklinksMap) -> ScrapsResult<()> {
        let render = ScrapRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(
            &self.base_url,
            self.timezone,
            &self.html_metadata,
            scrap_detail,
            backlinks_map,
        )
    }
}
