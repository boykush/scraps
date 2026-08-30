use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::site_nav::SiteNav;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::tag::Tag;
use scraps_libs::slugify;
use tera::Tera;

use crate::usecase::build::html::templates;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::tag::TagTera;

pub struct TagRender {
    tera: Tera,
    output_tags_dir_path: PathBuf,
}

impl TagRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<TagRender> {
        let tera = templates::tag(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(TagRender {
            tera,
            // Tag pages live in their own `tags/` directory, separate from
            // `scraps/`, to keep the two namespaces isolated (v1 design).
            output_tags_dir_path: output_dir_path.join("tags"),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
        site_nav: &SiteNav,
    ) -> ScrapsResult<()> {
        let mut context = templates::context(base_url, metadata);
        templates::insert_site_nav(&mut context, "", site_nav, backlinks_map);

        // insert to context for linked list
        context.insert("tag", &TagTera::new(tag, backlinks_map));

        let linked_scraps = backlinks_map.get_tag(tag);
        context.insert("linked_scraps", &LinkScrapsTera::new(&linked_scraps));

        // Build the slug-based path: `tags/<slug-segment>/<...>.html`. Each
        // segment of a hierarchical tag becomes a directory.
        let file_path = self
            .output_tags_dir_path
            .join(format!("{}.html", tag_slug_path(tag)));
        render_to_file(&self.tera, "__builtins/tag.html", &context, &file_path)
    }
}

fn tag_slug_path(tag: &Tag) -> String {
    tag.segments()
        .iter()
        .map(|s| slugify::by_dash(s))
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use crate::usecase::build::model::backlinks_map::BacklinksMap;
    use scraps_libs::lang::LangCode;
    use scraps_libs::model::base_url::BaseUrl;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::model::tags::Tags;
    use std::fs;
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
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true);
        // tag
        let tag1: Tag = "tag 1".into();

        // v1: tag pages live under `tags/` (not `scraps/`) and the slug is
        // built per-segment. "tag 1" slugifies to "tag-1".
        let tag1_html_path = output_dir_path.join("tags/tag-1.html");

        let render = TagRender::new(&static_dir_path, &output_dir_path).unwrap();

        render
            .run(&base_url, &metadata, &tag1, &backlinks_map, &site_nav)
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
        let site_nav = SiteNav::new(scraps.len(), Tags::new(&scraps), true);

        let tag: Tag = "ai/ml".into();
        // Expected path: public/tags/ai/ml.html
        let html_path = output_dir_path.join("tags/ai/ml.html");

        let render = TagRender::new(&static_dir_path, &output_dir_path).unwrap();
        render
            .run(&base_url, &metadata, &tag, &backlinks_map, &site_nav)
            .unwrap();

        let body = fs::read_to_string(html_path).unwrap();
        assert!(!body.is_empty());
    }
}
