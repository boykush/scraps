use std::fs;
use std::{fs::File, path::PathBuf};

use crate::build::cmd::HtmlMetadata;
use crate::build::model::sort::SortKey;
use crate::build::model::{linked_scraps_map::LinkedScrapsMap, scrap::Scrap};
use crate::libs::error::{ScrapError, ScrapResult};
use anyhow::Context;
use chrono_tz::Tz;

use crate::build::html::scrap_tera;

use super::serde::scrap::SerializeScrap;
use super::serde::scraps::SerializeScraps;

pub struct ScrapRender {
    static_dir_path: PathBuf,
    public_scraps_dir_path: PathBuf,
    scraps: Vec<Scrap>,
}

impl ScrapRender {
    pub fn new(
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
        scraps: &Vec<Scrap>,
    ) -> ScrapResult<ScrapRender> {
        let public_scraps_dir_path = &public_dir_path.join("scraps");
        fs::create_dir_all(public_scraps_dir_path).context(ScrapError::FileWrite)?;

        Ok(ScrapRender {
            static_dir_path: static_dir_path.to_owned(),
            public_scraps_dir_path: public_scraps_dir_path.to_owned(),
            scraps: scraps.to_owned(),
        })
    }

    pub fn run(
        &self,
        timezone: Tz,
        metadata: &HtmlMetadata,
        scrap: &Scrap,
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
        context.insert("scrap", &SerializeScrap::new(scrap, &linked_scraps_map));

        let linked_scraps = linked_scraps_map.linked_by(&scrap.title);
        context.insert(
            "linked_scraps",
            &SerializeScraps::new_with_sort(&linked_scraps, &linked_scraps_map, sort_key),
        );

        // render html
        let file_name = &format!("{}.html", scrap.title.slug);
        let wtr = File::create(self.public_scraps_dir_path.join(file_name))
            .context(ScrapError::FileWrite)?;
        tera.render_to("__builtins/scrap.html", &context, wtr)
            .context(ScrapError::PublicRender)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let timezone = chrono_tz::UTC;
        let metadata = HtmlMetadata::new(
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let sort_key = SortKey::CommittedDate;

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_scrap_htmls");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // scraps
        let scrap1 = &Scrap::new("scrap 1", "# header1", &None);
        let scrap2 = &Scrap::new("scrap 2", "[[scrap1]]", &None);
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let scrap1_html_path = public_dir_path.join(format!("scraps/{}.html", scrap1.title.slug));

        let render = ScrapRender::new(&static_dir_path, &public_dir_path, &scraps).unwrap();

        let result1 = render.run(timezone, &metadata, scrap1, &sort_key);
        assert!(result1.is_ok());

        let result2 = fs::read_to_string(scrap1_html_path);
        assert!(result2.is_ok());
    }
}
