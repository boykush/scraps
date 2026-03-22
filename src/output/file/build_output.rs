use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use scraps_libs::model::{base_url::BaseUrl, content::Content, scrap::Scrap, tag::Tag};

use crate::error::ScrapsResult;
use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    css::CssMetadata,
    html::HtmlMetadata,
    list_view_configs::ListViewConfigs,
    scrap_detail::{ScrapDetail, ScrapDetails},
};
use crate::usecase::build::port::{
    IndexPageWriter, ScrapPageWriter, SearchIndexWriter, StyleWriter, TagPageWriter,
};

use super::css::render::CSSRender;
use super::html::{
    index_render::IndexRender, scrap_render::ScrapRender, tag_render::TagRender,
    tags_index_render::TagsIndexRender,
};
use super::json::render::SearchIndexRender;

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

pub struct FileStyleWriter {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    css_metadata: CssMetadata,
}

impl FileStyleWriter {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path, css_metadata: CssMetadata) -> Self {
        Self {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
            css_metadata,
        }
    }
}

impl StyleWriter for FileStyleWriter {
    fn write(&self) -> ScrapsResult<()> {
        let render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        render.render_main(&self.css_metadata)
    }
}

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
