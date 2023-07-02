use std::fs;
use std::{fs::File, path::PathBuf};

use crate::build::model::scrap::Scrap;
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use chrono_tz::Tz;
use url::Url;

use crate::build::html::{
    scrap_tera,
    serde::{SerializeScrap, SerializeScraps},
};

pub struct IndexRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl IndexRender {
    pub fn new(static_dir_path: &PathBuf, public_dir_path: &PathBuf) -> ScrapResult<IndexRender> {
        fs::create_dir_all(&public_dir_path).context(ScrapError::FileWriteError)?;

        Ok(IndexRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        })
    }

    pub fn run(
        &self,
        timezone: &Tz,
        site_title: &str,
        site_description: &Option<String>,
        site_favicon: &Option<Url>,
        scraps: &Vec<Scrap>,
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            timezone,
            site_title,
            site_description,
            site_favicon,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;
        context.insert(
            "scraps",
            &SerializeScraps::new_with_sort(
                &scraps.iter().map(|s| SerializeScrap::new(&s)).collect(),
            ),
        );

        let template_name = if tera.get_template_names().any(|t| t == "index.html") {
            "index.html"
        } else {
            "__builtins/index.html"
        };

        let wtr = File::create(self.public_dir_path.join("index.html"))
            .context(ScrapError::PublicRenderError)?;
        tera.render_to(template_name, &context, wtr)
            .context(ScrapError::PublicRenderError)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::libs::resource::tests::FileResource;

    #[test]
    fn it_run() {
        // args
        let timezone = chrono_tz::UTC;
        let site_title = "Scrap";
        let site_description = Some("Scrap Wiki".to_string());
        let site_favicon = Some(Url::parse("https://github.io/image.png").unwrap());

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_index_html");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_html_path = static_dir_path.join("index.html");
        let resource_template_html = FileResource::new(&template_html_path);
        let resource_template_html_byte =
        "{% for scrap in scraps %}<a href=\"./{{ scrap.title }}.html\">{{ scrap.title }}</a>{% endfor %}"
        .as_bytes();

        // scraps
        let scrap1 = &Scrap::new("scrap1", "# header1", &Some(1));
        let scrap2 = &Scrap::new("scrap2", "## header2", &Some(0));
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let index_html_path = public_dir_path.join("index.html");

        resource_template_html.run(resource_template_html_byte, || {
            let render = IndexRender::new(&static_dir_path, &public_dir_path).unwrap();
            let result1 = render.run(
                &timezone,
                site_title,
                &site_description,
                &site_favicon,
                &scraps,
            );

            assert!(result1.is_ok());

            let result2 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result2,
                "<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );
        })
    }
}