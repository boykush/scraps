use std::fs;
use std::{fs::File, io::Write, path::PathBuf};

use crate::build::model::{scrap::Scrap, scraps::Scraps};
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use url::Url;

use crate::build::html::{
    content, scrap_tera,
    serde::{SScrap, SScraps},
};

pub struct HtmlRender {
    site_title: String,
    site_description: Option<String>,
    site_favicon: Option<Url>,
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
    scraps: Vec<Scrap>,
}

impl HtmlRender {
    pub fn new(
        site_title: &str,
        site_description: &Option<String>,
        site_favicon: &Option<Url>,
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
        scraps: &Vec<Scrap>,
    ) -> ScrapResult<HtmlRender> {
        fs::create_dir_all(&public_dir_path).context(ScrapError::FileWriteError)?;

        Ok(HtmlRender {
            site_title: site_title.to_owned(),
            site_description: site_description.to_owned(),
            site_favicon: site_favicon.to_owned(),
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
            scraps: scraps.to_vec(),
        })
    }

    pub fn render_index_html(&self) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            &self.site_title,
            &self.site_description,
            &self.site_favicon,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        let template_name = if tera.get_template_names().any(|t| t == "index.html") {
            "index.html"
        } else {
            "__builtins/index.html"
        };

        context.insert(
            "scraps",
            &SScraps(self.scraps.iter().map(|n| SScrap(n.clone())).collect()),
        );

        let wtr = File::create(self.public_dir_path.join("index.html"))
            .context(ScrapError::PublicRenderError)?;
        tera.render_to(template_name, &context, wtr)
            .context(ScrapError::PublicRenderError)
    }

    pub fn render_scrap_htmls(&self) -> ScrapResult<()> {
        self.scraps
            .iter()
            .map(|scrap| self.render_scrap_html(scrap))
            .collect::<ScrapResult<()>>()
    }

    fn render_scrap_html(&self, scrap: &Scrap) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            &self.site_title,
            &self.site_description,
            &self.site_favicon,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;

        context.insert("scrap", &SScrap(scrap.to_owned()));

        // insert to context for linked list
        let linked_map = Scraps::new(&self.scraps).gen_linked_map();
        let linked_list = linked_map.get(&scrap.title);
        if let Some(scraps) = linked_list {
            context.insert(
                "scraps",
                &SScraps(scraps.iter().map(|n| SScrap(n.to_owned())).collect()),
            );
        } else {
            context.insert("scraps", &Vec::<SScrap>::new())
        };

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
    use crate::libs::resource::tests::FileResource;

    #[test]
    fn it_render_index_html() {
        // args
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
        let scrap1 = &Scrap::new("scrap1", "# header1");
        let scrap2 = &Scrap::new("scrap2", "## header2");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let index_html_path = public_dir_path.join("index.html");

        resource_template_html.run(resource_template_html_byte, || {
            let render = HtmlRender::new(
                site_title,
                &site_description,
                &site_favicon,
                &static_dir_path,
                &public_dir_path,
                &scraps,
            )
            .unwrap();
            let result1 = render.render_index_html();

            assert!(result1.is_ok());

            let result2 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result2,
                "<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );
        })
    }

    #[test]
    fn it_render_scrap_htmls() {
        // args
        let site_title = "Scrap";
        let site_description = Some("Scrap Wiki".to_string());
        let site_favicon = Some(Url::parse("https://github.io/image.png").unwrap());

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_scrap_htmls");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // scraps
        let scrap1 = &Scrap::new("scrap1", "# header1");
        let scrap2 = &Scrap::new("scrap2", "[[scrap1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let scrap1_html_path = public_dir_path.join(format!("{}.html", scrap1.title));
        let scrap2_html_path = public_dir_path.join(format!("{}.html", scrap2.title));

        let render = HtmlRender::new(
            site_title,
            &site_description,
            &site_favicon,
            &static_dir_path,
            &public_dir_path,
            &scraps,
        )
        .unwrap();
        let result1 = render.render_scrap_htmls();

        assert!(result1.is_ok());

        let result2 = fs::read_to_string(scrap1_html_path);
        assert!(result2.is_ok());

        let result3 = fs::read_to_string(scrap2_html_path);
        assert!(result3.is_ok());
    }
}
