use std::collections::HashMap;
use std::path::Path;

use scraps_libs::model::{
    base_url::BaseUrl, content::Content, key::ScrapKey, scrap::Scrap, tag::Tag,
};

use crate::error::ScrapsResult;
use crate::service::search::render::SearchIndexRender;
use crate::usecase::build::{
    css::render::CSSRender,
    html::{
        about_render::AboutRender, index_render::IndexRender, scrap_render::ScrapRender,
        scraps_index_render::ScrapsIndexRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    model::{
        backlinks_map::BacklinksMap,
        css::CssMetadata,
        html::HtmlMetadata,
        list_view_configs::ListViewConfigs,
        scrap_detail::{ScrapDetail, ScrapDetails},
        site_nav::SiteNav,
    },
    renderer::{
        CssRenderer, HtmlAboutRenderer, HtmlIndexRenderer, HtmlScrapRenderer,
        HtmlScrapsIndexRenderer, HtmlTagRenderer, HtmlTagsIndexRenderer, SearchIndexJsonRenderer,
    },
};

/// Owns one render per page kind. Each carries a Tera instance whose
/// construction re-parses every template, so they are built once here rather
/// than per rendered page.
pub struct BuildRendererImpl {
    index_render: IndexRender,
    about_render: AboutRender,
    scrap_render: ScrapRender,
    scraps_index_render: ScrapsIndexRender,
    tags_index_render: TagsIndexRender,
    tag_render: TagRender,
    css_render: CSSRender,
    search_index_render: SearchIndexRender,
}

impl BuildRendererImpl {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<BuildRendererImpl> {
        Ok(BuildRendererImpl {
            index_render: IndexRender::new(static_dir_path, output_dir_path)?,
            about_render: AboutRender::new(static_dir_path, output_dir_path)?,
            scrap_render: ScrapRender::new(static_dir_path, output_dir_path)?,
            scraps_index_render: ScrapsIndexRender::new(static_dir_path, output_dir_path)?,
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
        site_nav: &SiteNav,
    ) -> ScrapsResult<usize> {
        self.index_render.run(
            base_url,
            html_metadata,
            list_view_configs,
            scrap_details,
            backlinks_map,
            site_nav,
        )
    }
}

impl HtmlAboutRenderer for BuildRendererImpl {
    fn render_about(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        readme_content: &Content,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.about_render.run(
            base_url,
            html_metadata,
            readme_content,
            backlinks_map,
            site_nav,
        )
    }
}

impl HtmlScrapRenderer for BuildRendererImpl {
    fn render_scrap(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
        scraps_by_key: &HashMap<ScrapKey, Scrap>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.scrap_render.run(
            base_url,
            html_metadata,
            scrap_detail,
            backlinks_map,
            scraps_by_key,
            site_nav,
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
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.scraps_index_render.run(
            base_url,
            html_metadata,
            scrap_details,
            backlinks_map,
            site_nav,
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
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.tags_index_render
            .run(base_url, html_metadata, scraps, backlinks_map, site_nav)
    }
}

impl HtmlTagRenderer for BuildRendererImpl {
    fn render_tag(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.tag_render
            .run(base_url, html_metadata, tag, backlinks_map, site_nav)
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
