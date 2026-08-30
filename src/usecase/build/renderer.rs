use std::collections::HashMap;

use scraps_libs::model::{
    base_url::BaseUrl, content::Content, key::ScrapKey, scrap::Scrap, tag::Tag,
};

use crate::error::ScrapsResult;
use crate::usecase::build::model::{
    backlinks_map::BacklinksMap,
    css::CssMetadata,
    html::HtmlMetadata,
    list_view_configs::ListViewConfigs,
    scrap_detail::{ScrapDetail, ScrapDetails},
    site_nav::SiteNav,
};

pub trait HtmlIndexRenderer {
    #[allow(clippy::too_many_arguments)]
    fn render_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        list_view_configs: &ListViewConfigs,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
        readme_content: &Option<Content>,
    ) -> ScrapsResult<usize>;
}

pub trait HtmlScrapRenderer {
    fn render_scrap(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scrap_detail: &ScrapDetail,
        backlinks_map: &BacklinksMap,
        scraps_by_key: &HashMap<ScrapKey, Scrap>,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()>;
}

pub trait HtmlScrapsIndexRenderer {
    fn render_scraps_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scrap_details: &ScrapDetails,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()>;
}

pub trait HtmlTagsIndexRenderer {
    fn render_tags_index(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()>;
}

pub trait HtmlTagRenderer {
    fn render_tag(
        &self,
        base_url: &BaseUrl,
        html_metadata: &HtmlMetadata,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
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
    + HtmlScrapsIndexRenderer
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
        + HtmlScrapsIndexRenderer
        + HtmlTagsIndexRenderer
        + HtmlTagRenderer
        + CssRenderer
        + SearchIndexJsonRenderer
        + Sync
{
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
            _site_nav: &SiteNav,
            _readme_content: &Option<Content>,
        ) -> ScrapsResult<usize> {
            Ok(1)
        }
    }

    impl HtmlScrapRenderer for BuildRendererTest {
        fn render_scrap(
            &self,
            _base_url: &BaseUrl,
            _html_metadata: &HtmlMetadata,
            _scrap_detail: &ScrapDetail,
            _backlinks_map: &BacklinksMap,
            _scraps_by_key: &HashMap<ScrapKey, Scrap>,
            _site_nav: &SiteNav,
        ) -> ScrapsResult<()> {
            Ok(())
        }
    }

    impl HtmlScrapsIndexRenderer for BuildRendererTest {
        fn render_scraps_index(
            &self,
            _base_url: &BaseUrl,
            _html_metadata: &HtmlMetadata,
            _scrap_details: &ScrapDetails,
            _backlinks_map: &BacklinksMap,
            _site_nav: &SiteNav,
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
            _site_nav: &SiteNav,
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
            _site_nav: &SiteNav,
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
