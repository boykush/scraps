use std::fs;
use std::{fs::File, path::PathBuf};

use crate::build::cmd::HtmlMetadata;
use crate::build::model::sort::SortKey;
use crate::build::model::tag::Tag;
use crate::build::model::{linked_scraps_map::LinkedScrapsMap, scrap::Scrap};
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use chrono_tz::Tz;

use crate::build::html::scrap_tera;

use super::serde::scraps::SerializeScraps;
use super::serde::tag::SerializeTag;

pub struct TagRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    scraps: Vec<Scrap>,
}

impl TagRender {
    pub fn new(
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
        scraps: &Vec<Scrap>,
    ) -> ScrapResult<TagRender> {
        fs::create_dir_all(&public_dir_path).context(ScrapError::FileWriteError)?;

        Ok(TagRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
            scraps: scraps.to_owned(),
        })
    }

    pub fn run(
        &self,
        timezone: &Tz,
        metadata: &HtmlMetadata,
        tag: &Tag,
        sort_key: &SortKey,
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            timezone,
            metadata,
            sort_key,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        // insert to context for linked list
        let linked_scraps_map = LinkedScrapsMap::new(&self.scraps);
        context.insert("tag", &SerializeTag::new(tag, &linked_scraps_map));

        let linked_scraps = linked_scraps_map.linked_by(&tag.title);
        context.insert(
            "linked_scraps",
            &SerializeScraps::new_with_sort(&linked_scraps, &linked_scraps_map, sort_key),
        );

        // render html
        let file_name = &format!("{}.html", tag.title.slug);
        let wtr = File::create(self.public_dir_path.join(file_name))
            .context(ScrapError::FileWriteError)?;
        tera.render_to("__builtins/tag.html", &context, wtr)
            .context(ScrapError::PublicRenderError)
    }
}

#[cfg(test)]
mod tests {
    use crate::build::model::title::Title;
    use url::Url;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let timezone = chrono_tz::UTC;
        let metadata = HtmlMetadata::new(
            &"Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let sort_key = SortKey::CommitedDate;

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_tag_htmls");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // scraps
        let scrap1 = &Scrap::new("scrap1", "[[tag1]]", &None);
        let scrap2 = &Scrap::new("scrap2", "[[tag1]][[tag2]]", &None);
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];
        // tag
        let tag1 = Tag::new(&Title::new("tag 1"));

        let tag1_html_path = public_dir_path.join(format!("{}.html", tag1.title.slug));

        let render = TagRender::new(&static_dir_path, &public_dir_path, &scraps).unwrap();

        let result1 = render.run(&timezone, &metadata, &tag1, &sort_key);
        assert!(result1.is_ok());

        let result2 = fs::read_to_string(tag1_html_path);
        assert!(result2.is_ok());
    }
}
