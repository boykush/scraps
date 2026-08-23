use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::build::model::html::HtmlMetadata;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::tags::Tags;
use tera::Tera;

use crate::usecase::build::html::templates;

use super::serde::tags::TagsTera;

pub struct TagsIndexRender {
    tera: Tera,
    output_tags_dir_path: PathBuf,
}

impl TagsIndexRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<TagsIndexRender> {
        let tera = templates::tags_index(&user_template_glob(static_dir_path, "*.html"))?;

        Ok(TagsIndexRender {
            tera,
            output_tags_dir_path: output_dir_path.join("tags"),
        })
    }

    pub fn run(
        &self,
        base_url: &BaseUrl,
        metadata: &HtmlMetadata,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
    ) -> ScrapsResult<()> {
        let mut context = templates::context(base_url, metadata);
        context.insert("tags", &TagsTera::new(&Tags::new(scraps), backlinks_map));

        let template_name =
            resolve_template(&self.tera, "tags_index.html", "__builtins/tags_index.html");
        let file_path = self.output_tags_dir_path.join("index.html");
        render_to_file(&self.tera, template_name, &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::build::model::backlinks_map::BacklinksMap;
    use rstest::rstest;
    use scraps_libs::{lang::LangCode, model::base_url::BaseUrl};
    use std::fs;
    use url::Url;

    use super::*;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Add static tags_index.html template
        project.add_static_file(
            "tags_index.html",
            b"{% for tag in tags %}<a href=\"./{{ tag.title }}.html\">{{ tag.title }}</a>{% endfor %}"
        );

        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let metadata = HtmlMetadata::new(
            &LangCode::default(),
            "Scrap",
            &Some("Scrap Wiki".to_string()),
            &Some(Url::parse("https://github.io/image.png").unwrap()),
        );

        // v1: tags come from explicit `#[[tag]]` declarations, not from
        // unresolved `[[]]` wikilinks.
        let scrap1 = Scrap::new("scrap1", &None, "#[[tag1]] #[[tag2]]");
        let scrap2 = Scrap::new("scrap2", &None, "#[[tag1]]");
        let scraps = vec![scrap1.to_owned(), scrap2.to_owned()];

        let backlinks_map = BacklinksMap::new(&scraps);

        let render = TagsIndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render
            .run(&base_url, &metadata, &scraps, &backlinks_map)
            .unwrap();

        let result1 = fs::read_to_string(project.output_path("tags/index.html")).unwrap();
        assert_eq!(
            result1,
            "<a href=\"./tag1.html\">tag1</a><a href=\"./tag2.html\">tag2</a>"
        );
    }
}
