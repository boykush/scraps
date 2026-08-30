use std::path::Path;

use chrono_tz::Tz;
use scraps_libs::model::{base_url::BaseUrl, content::Content, scrap::Scrap, tag::Tag};

use crate::error::ScrapsResult;
use crate::service::search::render::SearchIndexRender;
use crate::usecase::build::{
    css::render::CSSRender,
    html::{
        index_render::IndexRender, scrap_render::ScrapRender,
        scraps_index_render::ScrapsIndexRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender, title_index_render::TitleIndexRender,
    },
    model::{
        backlinks_map::BacklinksMap,
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_detail::{ScrapDetail, ScrapDetails},
    },
    renderer::{
        CssRenderer, HtmlIndexRenderer, HtmlScrapRenderer, HtmlScrapsIndexRenderer,
        HtmlTagRenderer, HtmlTagsIndexRenderer, HtmlTitleIndexRenderer, SearchIndexJsonRenderer,
    },
};

/// Owns one render per page kind. Each carries a Tera instance whose
/// construction re-parses every template, so they are built once here rather
/// than per rendered page.
pub struct BuildRendererImpl {
    index_render: IndexRender,
    scrap_render: ScrapRender,
    scraps_index_render: ScrapsIndexRender,
    title_index_render: TitleIndexRender,
    tags_index_render: TagsIndexRender,
    tag_render: TagRender,
    css_render: CSSRender,
    search_index_render: SearchIndexRender,
}

impl BuildRendererImpl {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<BuildRendererImpl> {
        Ok(BuildRendererImpl {
            index_render: IndexRender::new(static_dir_path, output_dir_path)?,
            scrap_render: ScrapRender::new(static_dir_path, output_dir_path)?,
            scraps_index_render: ScrapsIndexRender::new(static_dir_path, output_dir_path)?,
            title_index_render: TitleIndexRender::new(static_dir_path, output_dir_path)?,
            tags_index_render: TagsIndexRender::new(static_dir_path, output_dir_path)?,
            tag_render: TagRender::new(static_dir_path, output_dir_path)?,
            css_render: CSSRender::new(static_dir_path, output_dir_path)?,
            search_index_render: SearchIndexRender::new(static_dir_path, output_dir_path)?,
        })
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
        self.index_render.run(
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
        self.scrap_render.run(
            base_url,
            timezone,
            html_metadata,
            scrap_detail,
            backlinks_map,
        )
    }
}

impl HtmlScrapsIndexRenderer for BuildRendererImpl {
    fn render_scraps_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        self.scraps_index_render
            .run(base_url, html_metadata, scrap_details, backlinks_map)
    }
}

impl HtmlTitleIndexRenderer for BuildRendererImpl {
    fn render_title_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        self.title_index_render
            .run(base_url, html_metadata, scrap_details, backlinks_map)
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
        self.tags_index_render
            .run(base_url, html_metadata, scraps, backlinks_map)
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
        self.tag_render
            .run(base_url, html_metadata, tag, backlinks_map)
    }
}

impl CssRenderer for BuildRendererImpl {
    fn render_css(&self, css_metadata: &CssMetadata) -> ScrapsResult<()> {
        self.css_render.render_main(css_metadata)
    }
}

impl SearchIndexJsonRenderer for BuildRendererImpl {
    fn render_search_index(&self, base_url: &BaseUrl, scraps: &[Scrap]) -> ScrapsResult<()> {
        self.search_index_render.run(base_url, scraps)
    }
}
