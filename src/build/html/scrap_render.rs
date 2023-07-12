use std::fs;
use std::{fs::File, io::Write, path::PathBuf};

use crate::build::model::{scrap::Scrap, linked_scraps_map::LinkedScrapsMap};
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
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            timezone,
            site_title,
            site_description,
            site_favicon,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        context.insert("scrap", &SerializeScrap::new(&scrap));

        // insert to context for linked list
        let linked_list = LinkedScrapsMap::new(&self.scraps).linked_by(&scrap.title);
        context.insert(
            "scraps",
            &SerializeScraps::new_with_sort(
                &linked_list.iter().map(|s| SerializeScrap::new(&s)).collect(),
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
        );
        assert!(result1.is_ok());

        let result2 = fs::read_to_string(scrap1_html_path);
        assert!(result2.is_ok());
    }
}
