use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use scraps_libs::model::{base_url::BaseUrl, content::Content, scrap::Scrap, tag::Tag};

use crate::error::ScrapsResult;
use crate::service::search::render::SearchIndexRender;
use crate::usecase::build::{
    css::render::CSSRender,
    html::{
        index_render::IndexRender, scrap_render::ScrapRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    model::{
        backlinks_map::BacklinksMap,
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_detail::{ScrapDetail, ScrapDetails},
    },
    renderer::{
        CssRenderer, HtmlIndexRenderer, HtmlScrapRenderer, HtmlTagRenderer, HtmlTagsIndexRenderer,
        SearchIndexJsonRenderer,
    },
};

pub struct BuildRendererImpl {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl BuildRendererImpl {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path) -> BuildRendererImpl {
        BuildRendererImpl {
            static_dir_path: static_dir_path.to_path_buf(),
            public_dir_path: public_dir_path.to_path_buf(),
        }
    }
}

impl HtmlIndexRenderer for BuildRendererImpl {
    fn render_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize> {
        let index_render = IndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        index_render.run(
            base_url,
            html_metadata,
            list_view_configs,
            scrap_details,
            backlinks_map,
            readme_content,
        )
    }
}

impl HtmlScrapRenderer for BuildRendererImpl {
    fn render_scrap(
        &self,
        base_url: &BaseUrl,
        timezone: Tz,
        html_metadata: &HtmlMetadata,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let scrap_render = ScrapRender::new(&self.static_dir_path, &self.public_dir_path)?;
        scrap_render.run(
            base_url,
            timezone,
            html_metadata,
            scrap_detail,
            backlinks_map,
        )
    }
}

impl HtmlTagsIndexRenderer for BuildRendererImpl {
    fn render_tags_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let tags_index_render = TagsIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        tags_index_render.run(base_url, html_metadata, scraps, backlinks_map)
    }
}

impl HtmlTagRenderer for BuildRendererImpl {
    fn render_tag(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let tag_render = TagRender::new(&self.static_dir_path, &self.public_dir_path)?;
        tag_render.run(base_url, html_metadata, tag, backlinks_map)
    }
}

impl CssRenderer for BuildRendererImpl {
    fn render_css(&self, css_metadata: &CssMetadata) -> ScrapsResult<()> {
        let css_render = CSSRender::new(&self.static_dir_path, &self.public_dir_path);
        css_render.render_main(css_metadata)
    }
}

impl SearchIndexJsonRenderer for BuildRendererImpl {
    fn render_search_index(&self, base_url: &BaseUrl, scraps: &[Scrap]) -> ScrapsResult<()> {
        let search_index_render =
            SearchIndexRender::new(&self.static_dir_path, &self.public_dir_path)?;
        search_index_render.run(base_url, scraps)
    }
}
