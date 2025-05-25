use std::{fs::File, path::PathBuf};

use crate::error::BuildError;
use crate::error::{anyhow::Context, ScrapsResult};
use scraps_libs::model::scrap::Scrap;
use url::Url;

use super::search_index_tera;
use super::serde::search_index_scraps::SearchIndexScrapsTera;

pub struct SearchIndexRender {
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl SearchIndexRender {
    pub fn new(static_dir_path: &PathBuf, public_dir_path: &PathBuf) -> SearchIndexRender {
        SearchIndexRender {
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    pub fn run(&self, base_url: &Url, scraps: &[Scrap]) -> ScrapsResult<()> {
        let serialize_scraps = SearchIndexScrapsTera::new(scraps);

        Self::render_search_index_json(self, base_url, &serialize_scraps)
    }

    fn render_search_index_json(
        &self,
        base_url: &Url,
        scraps: &SearchIndexScrapsTera,
    ) -> ScrapsResult<()> {
        let (tera, mut context) = search_index_tera::base(
            base_url,
            self.static_dir_path.join("*.json").to_str().unwrap(),
        )?;
        let template_name = if tera.get_template_names().any(|t| t == "search_index.json") {
            "search_index.json"
        } else {
            "__builtins/search_index.json"
        };
        context.insert("scraps", scraps);
        let file_path = &self.public_dir_path.join("search_index.json");
        let wtr = File::create(file_path).context(BuildError::WriteFailure(file_path.clone()))?;
        tera.render_to(template_name, &context, wtr)
            .context(BuildError::WriteFailure(file_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use url::Url;

    use super::*;
    use scraps_libs::model::scrap::Scrap;
    use scraps_libs::tests::TestResources;

    #[test]
    fn it_run() {
        // args
        let base_url = Url::parse("http://localhost:1112/").unwrap();

        let test_resource_path =
            PathBuf::from("tests/resource/build/json/render/it_render_search_index_json");
        let static_dir_path = test_resource_path.join("static");
        let public_dir_path = test_resource_path.join("public");

        // static
        let template_json_path = static_dir_path.join("search_index.json");
        let resource_template_json_byte =
        "[{% for scrap in scraps %}{ \"title\": \"{{ scrap.link_title }}\", \"url\": \"{{ base_url}}scraps/{{ scrap.file_stem }}.html\" }{% if not loop.last %},{% endif %}{% endfor %}]"
        .as_bytes();

        // scraps
        let sc1 = Scrap::new("scrap1", &None, "# header1");
        let sc2 = Scrap::new("scrap2", &Some("Context"), "## header2");
        let scraps = vec![sc1, sc2];

        let search_index_json_path = public_dir_path.join("search_index.json");

        let mut test_resources = TestResources::new();
        test_resources
            .add_file(&template_json_path, resource_template_json_byte)
            .add_dir(&public_dir_path);
            
        test_resources.run(|| {
            let render = SearchIndexRender::new(&static_dir_path, &public_dir_path);
            render.run(&base_url, &scraps).unwrap();

            let result = fs::read_to_string(search_index_json_path).unwrap();
            assert_eq!(
                result,
                "[{ \"title\": \"scrap1\", \"url\": \"http://localhost:1112/scraps/scrap1.html\" },{ \"title\": \"Context/scrap2\", \"url\": \"http://localhost:1112/scraps/scrap2.context.html\" }]");
        });
    }
}
