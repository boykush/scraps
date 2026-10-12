use std::collections::HashMap;
use std::path::Path;

use scraps_libs::model::{base_url::BaseUrl, content::Content, key::ScrapKey, scrap::Scrap};

use crate::error::ScrapsResult;
use crate::service::search::render::SearchIndexRender;
use crate::service::tera_render::RenderOutput;
use crate::usecase::build::{
    css::render::CSSRender,
    fonts::FontsRender,
    html::{
        about_render::AboutRender, index_render::IndexRender, scrap_render::ScrapRender,
        scraps_index_render::ScrapsIndexRender, tag_render::TagRender,
        tags_index_render::TagsIndexRender,
    },
    js::JsRender,
    model::{
        backlinks_map::BacklinksMap, css::CssMetadata, html::HtmlMetadata,
        list_view_configs::ListViewConfigs, scrap_detail::ScrapDetails, site_nav::SiteNav,
    },
    renderer::{
        CssRenderer, FontsRenderer, HtmlAboutRenderer, HtmlIndexRenderer, HtmlScrapRenderer,
        HtmlScrapsIndexRenderer, HtmlTagRenderer, HtmlTagsIndexRenderer, JsRenderer,
        SearchIndexJsonRenderer,
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
    fonts_render: FontsRender,
    js_render: JsRender,
    search_index_render: SearchIndexRender,
}

impl BuildRendererImpl {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<BuildRendererImpl> {
        Self::with_output(
            static_dir_path,
            &RenderOutput::Dir(output_dir_path.to_path_buf()),
        )
    }

    /// Renders every page and keeps none, for checking that a build passes.
    pub fn discarding(static_dir_path: &Path) -> ScrapsResult<BuildRendererImpl> {
        Self::with_output(static_dir_path, &RenderOutput::Discard)
    }

    fn with_output(
        static_dir_path: &Path,
        output: &RenderOutput,
    ) -> ScrapsResult<BuildRendererImpl> {
        Ok(BuildRendererImpl {
            index_render: IndexRender::new(static_dir_path, output)?,
            about_render: AboutRender::new(static_dir_path, output)?,
            scrap_render: ScrapRender::new(static_dir_path, output)?,
            scraps_index_render: ScrapsIndexRender::new(static_dir_path, output)?,
            tags_index_render: TagsIndexRender::new(static_dir_path, output)?,
            tag_render: TagRender::new(static_dir_path, output)?,
            css_render: CSSRender::new(static_dir_path, output)?,
            fonts_render: FontsRender::new(output),
            js_render: JsRender::new(output),
            search_index_render: SearchIndexRender::new(static_dir_path, output)?,
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
    fn render_scraps(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        scraps_by_key: &HashMap<ScrapKey, &Scrap>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.scrap_render.run(
            base_url,
            html_metadata,
            scrap_details,
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
    fn render_tags(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        self.tag_render
            .run(base_url, html_metadata, backlinks_map, site_nav)
    }
}

impl CssRenderer for BuildRendererImpl {
    fn render_css(&self, css_metadata: &CssMetadata) -> ScrapsResult<()> {
        self.css_render.render_main(css_metadata)
    }
}

impl FontsRenderer for BuildRendererImpl {
    fn render_fonts(&self) -> ScrapsResult<()> {
        self.fonts_render.run()
    }
}

impl JsRenderer for BuildRendererImpl {
    fn render_js(&self) -> ScrapsResult<()> {
        self.js_render.run()
    }
}

impl SearchIndexJsonRenderer for BuildRendererImpl {
    fn render_search_index(&self, base_url: &BaseUrl, scraps: &[Scrap]) -> ScrapsResult<()> {
        self.search_index_render.run(base_url, scraps)
    }
}
