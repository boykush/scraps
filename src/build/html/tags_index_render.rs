use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::build::model::html::HtmlMetadata;
use crate::build::model::linked_scraps_map::LinkedScrapsMap;
use crate::error::{anyhow::Context, ScrapsError, ScrapsResult};
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::tags::Tags;
use url::Url;

use crate::build::html::tera::tags_index_tera;

use super::serde::tags::TagsTera;

pub struct TagsIndexRender {
    static_dir_path: PathBuf,
    public_tags_dir_path: PathBuf,
}

impl TagsIndexRender {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path) -> ScrapsResult<TagsIndexRender> {
        let public_tags_dir_path = &public_dir_path.join("tags");
        fs::create_dir_all(public_tags_dir_path).context(ScrapsError::FileWrite)?;

        Ok(TagsIndexRender {
            static_dir_path: static_dir_path.to_owned(),
            public_tags_dir_path: public_tags_dir_path.to_owned(),
        })
    }

    pub fn run(
        &self,
        base_url: &Url,
        metadata: &HtmlMetadata,
        scraps: &[Scrap],
    ) -> ScrapsResult<()> {
        let linked_scraps_map = LinkedScrapsMap::new(scraps);
        let stags = &TagsTera::new(&Tags::new(scraps), &linked_scraps_map);

        Self::render_html(self, base_url, metadata, stags)
    }

    fn render_html(
        &self,
        base_url: &Url,
        metadata: &HtmlMetadata,
        tags: &TagsTera,
    ) -> ScrapsResult<()> {
        let (tera, mut context) = tags_index_tera::base(
            base_url,
            metadata,
            self.static_dir_path.join("*.html").to_str().unwrap(),
        )?;
        let template_name = if tera.get_template_names().any(|t| t == "tags_index.html") {
            "tags_index.html"
        } else {
            "__builtins/tags_index.html"
        };
        context.insert("tags", tags);
        let wtr = File::create(self.public_tags_dir_path.join("index.html"))
            .context(ScrapsError::PublicRender)?;
        tera.render_to(template_name, &context, wtr)
            .context(ScrapsError::PublicRender)
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::lang::LangCode;
    use std::fs;
    use url::Url;

    use super::*;
    use scraps_libs::tests::FileResource;

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
            PathBuf::from("tests/resource/build/html/render/it_render_tags_index_html_1");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_html_path = static_dir_path.join("tags_index.html");
        let resource_template_html = FileResource::new(&template_html_path);
        let resource_template_html_byte =
        "{% for tag in tags %}<a href=\"./{{ tag.title }}.html\">{{ tag.title }}</a>{% endfor %}"
        .as_bytes();

        // scraps
        let scrap1 = &Scrap::new(&base_url, "scrap1", "[[tag1]][[tag2]]");
        let scrap2 = &Scrap::new(&base_url, "scrap2", "[[tag1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let index_html_path = public_dir_path.join("tags/index.html");

        resource_template_html.run(resource_template_html_byte, || {
            let render = TagsIndexRender::new(&static_dir_path, &public_dir_path).unwrap();
            let result1 = render.run(&base_url, &metadata, &scraps);

            assert!(result1.is_ok());

            let result2 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result2,
                "<a href=\"./tag1.html\">tag1</a><a href=\"./tag2.html\">tag2</a>"
            );
        })
    }
}
