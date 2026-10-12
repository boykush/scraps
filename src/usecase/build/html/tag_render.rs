use std::path::Path;

use crate::error::ScrapsResult;
use crate::service::tera_render::{RenderOutput, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::site_nav::SiteNav;
use rayon::prelude::*;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::page_path::PagePath;
use scraps_libs::model::tag::Tag;
use tera::Tera;
use tracing::{Level, span};

use crate::usecase::build::html::templates;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::tag::TagTera;

pub struct TagRender {
    tera: Tera,
    output: RenderOutput,
}

impl TagRender {
    pub fn new(static_dir_path: &Path, output: &RenderOutput) -> ScrapsResult<TagRender> {
        let tera = templates::tag(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(TagRender {
            tera,
            output: output.clone(),
        })
    }

    /// Renders a page for every tag in `site_nav`, all sharing one shell.
    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut shell = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut shell, "", site_nav, backlinks_map);

        site_nav.tags.iter().par_bridge().try_for_each(|tag| {
            let _span = span!(Level::INFO, "generate_html_tag").entered();
            self.render_page(&shell, tag, backlinks_map)
        })
    }

    fn render_page(
        &self,
        shell: &tera::Context,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let mut context = shell.clone();

        // insert to context for linked list
        context.insert("tag", &TagTera::new(tag, backlinks_map));

        let linked_scraps = backlinks_map.get_tag(tag);
        context.insert("linked_scraps", &LinkScrapsTera::new(linked_scraps));

        // Each segment of a hierarchical tag becomes a directory.
        self.output.render(
            &self.tera,
            "__builtins/tag.html",
            &context,
            PagePath::of_tag(tag).as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::usecase::build::model::backlinks_map::BacklinksMap;
    use crate::usecase::build::model::scrap_types::ScrapTypes;
    use scraps_libs::lang::LangCode;
    use scraps_libs::model::base_url::BaseUrl;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::model::tags::Tags;
    use std::fs;
    use std::path::PathBuf;
    use url::Url;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_tag_htmls");
        let static_dir_path = test_resource_path.join("static");
        let output_dir_path = test_resource_path.join("_site");

        // scraps with explicit `#[[tag]]` tags
        let scrap1 = &Scrap::new("scrap1", &None, "#[[tag 1]]");
        let scrap2 = &Scrap::new("scrap2", &None, "#[[tag 1]] #[[tag2]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        let tag1_html_path = output_dir_path.join("tags/tag 1.html");

        let render = TagRender::new(
            &static_dir_path,
            &RenderOutput::Dir(output_dir_path.clone()),
        )
        .unwrap();

        render
            .run(&base_url, &metadata, &backlinks_map, &site_nav)
            .unwrap();

        let result2 = fs::read_to_string(tag1_html_path).unwrap();
        assert!(!result2.is_empty());
    }

    #[test]
    fn it_run_hierarchical_creates_nested_directory() {
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_nested_tag_htmls");
        let static_dir_path = test_resource_path.join("static");
        let output_dir_path = test_resource_path.join("_site");

        let scrap = Scrap::new("paper", &None, "#[[ai/ml]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let site_nav = SiteNav::new(
            scraps.len(),
            Tags::new(&scraps),
            ScrapTypes::new(&scraps),
            true,
            chrono_tz::UTC,
            false,
        );

        // Expected path: public/tags/ai/ml.html
        let html_path = output_dir_path.join("tags/ai/ml.html");

        let render = TagRender::new(
            &static_dir_path,
            &RenderOutput::Dir(output_dir_path.clone()),
        )
        .unwrap();
        render
            .run(&base_url, &metadata, &backlinks_map, &site_nav)
            .unwrap();

        let body = fs::read_to_string(html_path).unwrap();
        assert!(!body.is_empty());
    }
}
