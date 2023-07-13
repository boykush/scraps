use std::fs;
use std::{fs::File, io::Write, path::PathBuf};

use crate::build::model::sort::SortKey;
use crate::build::model::{linked_scraps_map::LinkedScrapsMap, scrap::Scrap};
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use chrono_tz::Tz;
use url::Url;

use crate::build::html::{
    content, scrap_tera,
    serde::{SerializeScrap, SerializeScraps},
};

pub struct ScrapRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    scraps: Vec<Scrap>,
}

impl ScrapRender {
    pub fn new(
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
        scraps: &Vec<Scrap>,
    ) -> ScrapResult<ScrapRender> {
        fs::create_dir_all(&public_dir_path).context(ScrapError::FileWriteError)?;

        Ok(ScrapRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
            scraps: scraps.to_owned(),
        })
    }

    pub fn run(
        &self,
        timezone: &Tz,
        site_title: &str,
        site_description: &Option<String>,
        site_favicon: &Option<Url>,
        scrap: &Scrap,
        sort_key: &SortKey,
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            timezone,
            site_title,
            site_description,
            site_favicon,
            sort_key,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        // insert to context for linked list
        let linked_scraps_map = LinkedScrapsMap::new(&self.scraps);
        context.insert("scrap", &SerializeScrap::new(&scrap, &linked_scraps_map));

        let linked_scraps = linked_scraps_map.linked_by(&scrap.title);
        context.insert(
            "linked_scraps",
            &SerializeScraps::new_with_sort(
                &linked_scraps
                    .iter()
                    .map(|s| SerializeScrap::new(&s, &linked_scraps_map))
                    .collect(),
                sort_key,
            ),
        );

        // render html
        let rendered = tera
            .render("__builtins/scrap.html", &context)
            .context(ScrapError::PublicRenderError)?;
        let html = content::insert(&rendered, &scrap.html_content);

        // write
        let file_name = &format!("{}.html", scrap.title);
        let mut wtr = File::create(self.public_dir_path.join(file_name))
            .context(ScrapError::FileWriteError)?;
        wtr.write_all(html.as_bytes())
            .context(ScrapError::FileWriteError)?;
        wtr.flush().context(ScrapError::FileWriteError)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let timezone = chrono_tz::UTC;
        let site_title = "Scrap";
        let site_description = Some("Scrap Wiki".to_string());
        let site_favicon = Some(Url::parse("https://github.io/image.png").unwrap());
        let sort_key = SortKey::CommitedDate;

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_scrap_htmls");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // scraps
        let scrap1 = &Scrap::new("scrap1", "# header1", &None);
        let scrap2 = &Scrap::new("scrap2", "[[scrap1]]", &None);
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let scrap1_html_path = public_dir_path.join(format!("{}.html", scrap1.title));

        let render = ScrapRender::new(&static_dir_path, &public_dir_path, &scraps).unwrap();

        let result1 = render.run(
            &timezone,
            site_title,
            &site_description,
            &site_favicon,
            scrap1,
            &sort_key,
        );
        assert!(result1.is_ok());

        let result2 = fs::read_to_string(scrap1_html_path);
        assert!(result2.is_ok());
    }
}
