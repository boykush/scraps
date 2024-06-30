use std::fs;
use std::{fs::File, path::PathBuf};

use crate::build::cmd::HtmlMetadata;
use crate::build::model::linked_scraps_map::LinkedScrapsMap;
use crate::build::model::paging::Paging;
use crate::build::model::scrap::Scrap;
use crate::build::model::sort::SortKey;
use crate::build::model::tags::Tags;
use crate::libs::error::{error::ScrapError, result::ScrapResult};
use anyhow::Context;
use chrono_tz::Tz;

use crate::build::html::scrap_tera;

use super::page_pointer::PagePointer;
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
        paging: &Paging,
    ) -> ScrapResult<()> {
        let linked_scraps_map = LinkedScrapsMap::new(scraps);
        let sorted_scraps =
            SerializeScraps::new_with_sort(&scraps.to_vec(), &linked_scraps_map, sort_key);
        let paginated = sorted_scraps
            .chunks(paging.size_with(scraps))
            .into_iter()
            .enumerate();
        let last_page_num = paginated.len();
        let paginated_with_pointer = paginated.map(|(idx, paginated_scraps)| {
            let page_num = idx + 1;
            let pointer = PagePointer::new(page_num, last_page_num);
            (pointer, paginated_scraps)
        });
        let stags = &SerializeTags::new(&Tags::new(scraps), &linked_scraps_map);

        paginated_with_pointer
            .map(|(pointer, paginated_scraps)| {
                Self::render_paginated_html(
                    &self,
                    timezone,
                    metadata,
                    sort_key,
                    stags,
                    &paginated_scraps,
                    &pointer,
                )
            })
            .collect::<ScrapResult<()>>()
    }

    fn render_paginated_html(
        &self,
        timezone: &Tz,
        metadata: &HtmlMetadata,
        sort_key: &SortKey,
        tags: &SerializeTags,
        paginated_scraps: &SerializeScraps,
        pointer: &PagePointer,
    ) -> ScrapResult<()> {
        let (tera, mut context) = scrap_tera::init(
            timezone,
            metadata,
            sort_key,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;
        let template_name = if tera.get_template_names().any(|t| t == "index.html") {
            "index.html"
        } else {
            "__builtins/index.html"
        };
        context.insert("scraps", &paginated_scraps);
        if pointer.is_index() {
            context.insert("tags", tags);
        };
        context.insert("prev", &pointer.prev);
        context.insert("next", &pointer.next);
        let wtr = File::create(self.public_dir_path.join(&pointer.current_file_name()))
            .context(ScrapError::PublicRenderError)?;
        tera.render_to(template_name, &context, wtr)
            .context(ScrapError::PublicRenderError)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

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
        let sort_key = SortKey::CommittedDate;
        let paging = Paging::Not;

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_index_html_1");
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
            let result1 = render.run(&timezone, &metadata, &scraps, &sort_key, &paging);

            assert!(result1.is_ok());

            let result2 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result2,
                "<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );
        })
    }

    #[test]
    fn it_run_paging() {
        // args
        let timezone = chrono_tz::UTC;
        let metadata = HtmlMetadata::new(
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );
        let sort_key = SortKey::CommittedDate;
        let paging = Paging::By(2);

        let test_resource_path =
            PathBuf::from("tests/resource/build/html/render/it_render_index_html_2");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_html_path = static_dir_path.join("index.html");
        let resource_template_html = FileResource::new(&template_html_path);
        let resource_template_html_byte =
        "{% for scrap in scraps %}<a href=\"./{{ scrap.title }}.html\">{{ scrap.title }}</a>{% endfor %}"
        .as_bytes();

        // scraps
        let scrap1 = &Scrap::new("scrap1", "# header1", &Some(3));
        let scrap2 = &Scrap::new("scrap2", "## header2", &Some(2));
        let scrap3 = &Scrap::new("scrap3", "### header3", &Some(1));
        let scrap4 = &Scrap::new("scrap4", "#### header4", &Some(0));
        let scraps = vec![
            scrap1.to_owned(),
            scrap2.to_owned(),
            scrap3.to_owned(),
            scrap4.to_owned(),
        ];

        let index_html_path = public_dir_path.join("index.html");
        let page2_html_path = public_dir_path.join("2.html");

        resource_template_html.run(resource_template_html_byte, || {
            let render = IndexRender::new(&static_dir_path, &public_dir_path).unwrap();
            let result1 = render.run(&timezone, &metadata, &scraps, &sort_key, &paging);

            assert!(result1.is_ok());

            let result2 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result2,
                "<a href=\"./scrap1.html\">scrap1</a><a href=\"./scrap2.html\">scrap2</a>"
            );

            let result3 = fs::read_to_string(page2_html_path).unwrap();
            assert_eq!(
                result3,
                "<a href=\"./scrap3.html\">scrap3</a><a href=\"./scrap4.html\">scrap4</a>"
            );
        })
    }
}
