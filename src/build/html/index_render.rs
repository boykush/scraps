use std::fs;
use std::{fs::File, path::PathBuf};

use crate::build::cmd::HtmlMetadata;
use crate::build::model::linked_scraps_map::LinkedScrapsMap;
use crate::build::model::scrap::Scrap;
use crate::build::model::sort::SortKey;
use crate::build::model::tags::Tags;
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use chrono_tz::Tz;

use crate::build::html::scrap_tera;

use super::serde::scraps::SerializeScraps;
use super::serde::tags::SerializeTags;

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
        metadata: &HtmlMetadata,
        scraps: &Vec<Scrap>,
        sort_key: &SortKey,
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            timezone,
            metadata,
            sort_key,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;
        let linked_scraps_map = LinkedScrapsMap::new(scraps);
        context.insert(
            "scraps",
            &SerializeScraps::new_with_sort(&scraps, &linked_scraps_map, sort_key),
        );
        let tags = Tags::new(scraps);
        context.insert(
            "tags",
            &SerializeTags::new(&tags.values.iter().cloned().collect(), &linked_scraps_map),
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
        let metadata = HtmlMetadata::new(
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let sort_key = SortKey::CommitedDate;

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
            let result1 = render.run(&timezone, &metadata, &scraps, &sort_key);

            assert!(result1.is_ok());

            let result2 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result2,
                "<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );
        })
    }
}
