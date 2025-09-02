use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::tags::Tags;

use crate::usecase::build::html::tera::tags_index_tera;

use super::serde::tags::TagsTera;

pub struct TagsIndexRender {
    static_dir_path: PathBuf,
    public_tags_dir_path: PathBuf,
}

impl TagsIndexRender {
    pub fn new(static_dir_path: &Path, public_dir_path: &Path) -> ScrapsResult<TagsIndexRender> {
        let public_tags_dir_path = &public_dir_path.join("tags");
        fs::create_dir_all(public_tags_dir_path).context(BuildError::CreateDir)?;

        Ok(TagsIndexRender {
            static_dir_path: static_dir_path.to_owned(),
            public_tags_dir_path: public_tags_dir_path.to_owned(),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        scraps: &[Scrap],
    ) -> ScrapsResult<()> {
        let backlinks_map = BacklinksMap::new(scraps);
        let stags = &TagsTera::new(&Tags::new(scraps), &backlinks_map);

        Self::render_html(self, base_url, metadata, stags)
    }

    fn render_html(
        &self,
        base_url: &BaseUrl,
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
        let file_path = &self.public_tags_dir_path.join("index.html");
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to(template_name, &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::{lang::LangCode, model::base_url::BaseUrl, tests::TestResources};
    use std::fs;
    use url::Url;

    use super::*;

    #[test]
    fn it_run() {
        // args
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
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
        let resource_template_html_byte =
        "{% for tag in tags %}<a href=\"./{{ tag.title }}.html\">{{ tag.title }}</a>{% endfor %}"
        .as_bytes();

        // scraps
        let scrap1 = Scrap::new("scrap1", &None, "[[tag1]][[tag2]]");
        let scrap2 = Scrap::new("scrap2", &None, "[[tag1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let index_html_path = public_dir_path.join("tags/index.html");

        let mut test_resources = TestResources::new();
        test_resources.add_file(&template_html_path, resource_template_html_byte);

        test_resources.run(|| {
            let render = TagsIndexRender::new(&static_dir_path, &public_dir_path).unwrap();
            render.run(&base_url, &metadata, &scraps).unwrap();

            let result1 = fs::read_to_string(index_html_path).unwrap();
            assert_eq!(
                result1,
                "<a href=\"./tag1.html\">tag1</a><a href=\"./tag2.html\">tag2</a>"
            );
        })
    }
}
