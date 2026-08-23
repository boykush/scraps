use std::path::{Path, PathBuf};

use crate::error::ScrapsResult;
use crate::service::tera_render::{render_to_file, resolve_template, user_template_glob};
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::scrap::Scrap;
use tera::Tera;

use super::search_index_tera;
use super::serde::search_index_scraps::SearchIndexScrapsTera;

pub struct SearchIndexRender {
    tera: Tera,
    output_dir_path: PathBuf,
}

impl SearchIndexRender {
    pub fn new(static_dir_path: &Path, output_dir_path: &Path) -> ScrapsResult<SearchIndexRender> {
        let tera = search_index_tera::tera(&user_template_glob(static_dir_path, "*.json"))?;

        Ok(SearchIndexRender {
            tera,
            output_dir_path: output_dir_path.to_path_buf(),
        })
    }

    pub fn run(&self, base_url: &BaseUrl, scraps: &[Scrap]) -> ScrapsResult<()> {
        let mut context = search_index_tera::context(base_url);
        context.insert("scraps", &SearchIndexScrapsTera::new(scraps));

        let template_name = resolve_template(
            &self.tera,
            "search_index.json",
            "__builtins/search_index.json",
        );
        let file_path = self.output_dir_path.join("search_index.json");
        render_to_file(&self.tera, template_name, &context, &file_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;
    use std::fs;
    use url::Url;

    use super::*;
    use scraps_libs::model::scrap::Scrap;

    #[rstest]
    fn it_run(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Add static search_index.json template
        project.add_static_file(
            "search_index.json",
            b"[{% for scrap in scraps %}{ \"title\": \"{{ scrap.link_title }}\", \"url\": \"{{ base_url}}scraps/{{ scrap.file_stem }}.html\" }{% if not loop.last %},{% endif %}{% endfor %}]"
        );

        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();

        // Create scraps
        let sc1 = Scrap::new("scrap1", &None, "# header1");
        let sc2 = Scrap::new("scrap2", &Some("Context".into()), "## header2");
        let scraps = vec![sc1, sc2];

        let render = SearchIndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render.run(&base_url, &scraps).unwrap();

        let result = fs::read_to_string(project.output_path("search_index.json")).unwrap();
        assert_eq!(
            result,
            "[{ \"title\": \"scrap1\", \"url\": \"http://localhost:1112/scraps/scrap1.html\" },{ \"title\": \"Context/scrap2\", \"url\": \"http://localhost:1112/scraps/context/scrap2.html\" }]");
    }

    #[rstest]
    fn it_run_builtin_emits_valid_json(#[from(temp_scrap_project)] project: TempScrapProject) {
        // No static template, so the builtin one is rendered.
        let base_url = BaseUrl::new(Url::parse("http://localhost:1112/").unwrap()).unwrap();
        let scraps = vec![
            Scrap::new("scrap1", &None, "# header1"),
            Scrap::new("scrap2", &Some("Context".into()), "## header2"),
            Scrap::new("quote\"in title", &None, "# header3"),
        ];

        let render = SearchIndexRender::new(&project.static_dir, &project.output_dir).unwrap();
        render.run(&base_url, &scraps).unwrap();

        let result = fs::read_to_string(project.output_path("search_index.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let entries = parsed.as_array().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["title"], "scrap1");
        assert_eq!(
            entries[0]["url"],
            "http://localhost:1112/scraps/scrap1.html"
        );
        assert_eq!(entries[1]["title"], "Context/scrap2");
        assert_eq!(
            entries[1]["url"],
            "http://localhost:1112/scraps/context/scrap2.html"
        );
        assert_eq!(entries[2]["title"], "quote\"in title");
    }
}
