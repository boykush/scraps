use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::slug::Slug;
use scraps_libs::model::tag::Tag;
use url::Url;

use crate::usecase::build::html::tera::tag_tera;

use super::serde::link_scraps::LinkScrapsTera;
use super::serde::tag::TagTera;

pub struct TagRender {
    static_dir_path: PathBuf,
    public_scraps_dir_path: PathBuf,
    scraps: Vec<Scrap>,
}

impl TagRender {
    pub fn new(
        static_dir_path: &Path,
        public_dir_path: &Path,
        scraps: &Vec<Scrap>,
    ) -> ScrapsResult<TagRender> {
        let public_tags_dir_path = &public_dir_path.join("scraps");
        fs::create_dir_all(public_tags_dir_path).context(BuildError::CreateDir)?;

        Ok(TagRender {
            static_dir_path: static_dir_path.to_owned(),
            public_scraps_dir_path: public_tags_dir_path.to_owned(),
            scraps: scraps.to_owned(),
        })
    }

    pub fn run(&self, base_url: &Url, metadata: &HtmlMetadata, tag: &Tag) -> ScrapsResult<()> {
        let (tera, mut context) = tag_tera::base(
            base_url,
            metadata,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        // insert to context for linked list
        let backlinks_map = BacklinksMap::new(&self.scraps);
        context.insert("tag", &TagTera::new(tag, &backlinks_map));

        let linked_scraps = backlinks_map.get(&tag.title.clone().into());
        context.insert(
            "linked_scraps",
            &LinkScrapsTera::new(&linked_scraps, base_url),
        );

        // render html
        let file_name = &format!("{}.html", Slug::from(tag.title.clone()));
        let file_path = &self.public_scraps_dir_path.join(file_name);
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to("__builtins/tag.html", &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::lang::LangCode;
    use url::Url;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let base_url = Url::parse("http://localhost:1112/").unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_tag_htmls");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // scraps
        let scrap1 = &Scrap::new(&base_url, "scrap1", &None, "[[tag1]]");
        let scrap2 = &Scrap::new(&base_url, "scrap2", &None, "[[tag1]][[tag2]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];
        // tag
        let tag1: Tag = "tag 1".into();

        let tag1_html_path =
            public_dir_path.join(format!("scraps/{}.html", Slug::from(tag1.title.clone())));

        let render = TagRender::new(&static_dir_path, &public_dir_path, &scraps).unwrap();

        render.run(&base_url, &metadata, &tag1).unwrap();

        let result2 = fs::read_to_string(tag1_html_path).unwrap();
        assert!(!result2.is_empty());
    }
}
