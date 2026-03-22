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

pub struct FileBuildOutput {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    base_url: BaseUrl,
    timezone: Tz,
    html_metadata: HtmlMetadata,
    css_metadata: CssMetadata,
}

impl FileBuildOutput {
    pub fn new(
        static_dir_path: &Path,
        public_dir_path: &Path,
        base_url: BaseUrl,
        timezone: Tz,
        html_metadata: HtmlMetadata,
        css_metadata: CssMetadata,
    ) -> Self {
        FileBuildOutput {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
            base_url,
            timezone,
            html_metadata,
            css_metadata,
        }
    }
}

impl IndexPageWriter for FileBuildOutput {
    fn write_index_page(
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

impl ScrapPageWriter for FileBuildOutput {
    fn write_scrap_page(
        &self,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
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

impl TagPageWriter for FileBuildOutput {
    fn write_tags_index_page(
        &self,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let render = TagsIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(&self.base_url, &self.html_metadata, scraps, backlinks_map)
    }

    fn write_tag_page(&self, tag: &Tag, backlinks_map: &BacklinksMap) -> ScrapsResult<()> {
        let render = TagRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(&self.base_url, &self.html_metadata, tag, backlinks_map)
    }
}

impl StyleWriter for FileBuildOutput {
    fn write_style(&self) -> ScrapsResult<()> {
        let render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        render.render_main(&self.css_metadata)
    }
}

impl SearchIndexWriter for FileBuildOutput {
    fn write_search_index(&self, scraps: &[Scrap]) -> ScrapsResult<()> {
        let render = SearchIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        render.run(&self.base_url, scraps)
    }
}
