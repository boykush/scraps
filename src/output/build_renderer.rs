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
};

pub trait HtmlIndexRenderer {
    fn render_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize>;
}

pub trait HtmlScrapRenderer {
    fn render_scrap(
        &self,
        base_url: &BaseUrl,
        timezone: Tz,
        html_metadata: &HtmlMetadata,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()>;
}

pub trait HtmlTagsIndexRenderer {
    fn render_tags_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()>;
}

pub trait HtmlTagRenderer {
    fn render_tag(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()>;
}

pub trait CssRenderer {
    fn render_css(&self, css_metadata: &CssMetadata) -> ScrapsResult<()>;
}

pub trait SearchIndexJsonRenderer {
    fn render_search_index(&self, base_url: &BaseUrl, scraps: &[Scrap]) -> ScrapsResult<()>;
}

pub trait BuildRenderer:
    HtmlIndexRenderer
    + HtmlScrapRenderer
    + HtmlTagsIndexRenderer
    + HtmlTagRenderer
    + CssRenderer
    + SearchIndexJsonRenderer
    + Sync
{
}

impl<T> BuildRenderer for T where
    T: HtmlIndexRenderer
        + HtmlScrapRenderer
        + HtmlTagsIndexRenderer
        + HtmlTagRenderer
        + CssRenderer
        + SearchIndexJsonRenderer
        + Sync
{
}

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

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct BuildRendererTest();

    impl BuildRendererTest {
        pub fn new() -> Self {
            Self()
        }
    }

    impl HtmlIndexRenderer for BuildRendererTest {
        fn render_index(
            &self,
            _base_url: &BaseUrl,
            _html_metadata: &HtmlMetadata,
            _list_view_configs: &ListViewConfigs,
            _scrap_details: &ScrapDetails,
            _backlinks_map: &BacklinksMap,
            _readme_content: &Option<Content>,
        ) -> ScrapsResult<usize> {
            Ok(1)
        }
    }

    impl HtmlScrapRenderer for BuildRendererTest {
        fn render_scrap(
            &self,
            _base_url: &BaseUrl,
            _timezone: Tz,
            _html_metadata: &HtmlMetadata,
            _scrap_detail: &ScrapDetail,
            _backlinks_map: &BacklinksMap,
        ) -> ScrapsResult<()> {
            Ok(())
        }
    }

    impl HtmlTagsIndexRenderer for BuildRendererTest {
        fn render_tags_index(
            &self,
            _base_url: &BaseUrl,
            _html_metadata: &HtmlMetadata,
            _scraps: &[Scrap],
            _backlinks_map: &BacklinksMap,
        ) -> ScrapsResult<()> {
            Ok(())
        }
    }

    impl HtmlTagRenderer for BuildRendererTest {
        fn render_tag(
            &self,
            _base_url: &BaseUrl,
            _html_metadata: &HtmlMetadata,
            _tag: &Tag,
            _backlinks_map: &BacklinksMap,
        ) -> ScrapsResult<()> {
            Ok(())
        }
    }

    impl CssRenderer for BuildRendererTest {
        fn render_css(&self, _css_metadata: &CssMetadata) -> ScrapsResult<()> {
            Ok(())
        }
    }

    impl SearchIndexJsonRenderer for BuildRendererTest {
        fn render_search_index(&self, _base_url: &BaseUrl, _scraps: &[Scrap]) -> ScrapsResult<()> {
            Ok(())
        }
    }
}
