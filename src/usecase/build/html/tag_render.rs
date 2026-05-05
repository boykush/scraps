use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::tag::Tag;
use scraps_libs::slugify;

use crate::usecase::build::html::tera::tag_tera;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::tag::TagTera;

pub struct TagRender {
    static_dir_path: PathBuf,
    output_tags_dir_path: PathBuf,
}

impl TagRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<TagRender> {
        // Tag pages live in their own `tags/` directory, separate from
        // `scraps/`, to keep the two namespaces isolated (v1 design).
        let output_tags_dir_path = &output_dir_path.join("tags");
        fs::create_dir_all(output_tags_dir_path).context(BuildError::CreateDir)?;

        Ok(TagRender {
            static_dir_path: static_dir_path.to_owned(),
            output_tags_dir_path: output_tags_dir_path.to_owned(),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        tag: &Tag,
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let (tera, mut context) = tag_tera::base(
            base_url,
            metadata,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        // insert to context for linked list
        context.insert("tag", &TagTera::new(tag, backlinks_map));

        let linked_scraps = backlinks_map.get_tag(tag);
        context.insert(
            "linked_scraps",
            &LinkScrapsTera::new(&linked_scraps, base_url),
        );

        // Build the slug-based path: `tags/<slug-segment>/<...>.html`. Each
        // segment of a hierarchical tag becomes a directory.
        let slug_path = tag_slug_path(tag);
        let file_path = self
            .output_tags_dir_path
            .join(format!("{}.html", slug_path));
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).context(BuildError::CreateDir)?;
        }
        let wtr = File::create(&file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to("__builtins/tag.html", &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
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
        // tag
        let tag1: Tag = "tag 1".into();

        // v1: tag pages live under `tags/` (not `scraps/`) and the slug is
        // built per-segment. "tag 1" slugifies to "tag-1".
        let tag1_html_path = output_dir_path.join("tags/tag-1.html");

        let render = TagRender::new(&static_dir_path, &output_dir_path).unwrap();

        render
            .run(&base_url, &metadata, &tag1, &backlinks_map)
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

        let tag: Tag = "ai/ml".into();
        // Expected path: public/tags/ai/ml.html
        let html_path = output_dir_path.join("tags/ai/ml.html");

        let render = TagRender::new(&static_dir_path, &output_dir_path).unwrap();
        render
            .run(&base_url, &metadata, &tag, &backlinks_map)
            .unwrap();

        let body = fs::read_to_string(html_path).unwrap();
        assert!(!body.is_empty());
    }
}
