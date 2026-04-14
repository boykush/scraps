use std::path::Path;

use itertools::Itertools;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::display::tag::{DisplayTag, DisplayTagTable};
use crate::cli::json::tag::TagJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::tag::list::usecase::ListTagUsecase;

pub fn run(json: bool, project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);

    let scraps = read_scraps::to_all_scraps(&scraps_dir_path)?;
    let usecase = ListTagUsecase::new();

    let (tags, backlinks_map) = usecase.execute(&scraps)?;

    if json {
        let tags_json: Vec<TagJson> = tags
            .into_iter()
            .map(|tag| {
                let backlinks_count = backlinks_map.get(&tag.title().clone().into()).len();
                TagJson {
                    title: tag.title().to_string(),
                    backlinks_count,
                }
            })
            .sorted_by(|a, b| b.backlinks_count.cmp(&a.backlinks_count))
            .collect();
        println!("{}", serde_json::to_string(&tags_json)?);
        Ok(())
    } else {
        let base_url = config.get_base_url();
        let display_tags_result = tags
            .into_iter()
            .map(|tag| DisplayTag::new(&tag, base_url.as_ref(), &backlinks_map))
            .collect::<ScrapsResult<Vec<DisplayTag>>>();

        display_tags_result.map(|tags| {
            let sorted = tags
                .into_iter()
                .sorted_by_key(|tag| tag.backlinks_count())
                .rev()
                .collect::<Vec<_>>();
            let table = DisplayTagTable::new(sorted);
            println!("{table}");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn run_succeeds_with_tags_and_base_url(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"[ssg]\nbase_url = \"https://example.com/\"\ntitle = \"Test\"")
            .add_scrap("a.md", b"#[[tag1]]")
            .add_scrap("b.md", b"#[[tag1]] #[[tag2]]");

        let result = run(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_without_ssg_section(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("a.md", b"#[[tag1]]");

        let result = run(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_with_empty_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"");

        let result = run(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let result = run(false, Some(project.project_root.as_path()));
        assert!(result.is_err());
    }

    #[rstest]
    fn run_json_succeeds_with_tags(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"#[[tag1]]")
            .add_scrap("b.md", b"#[[tag1]] #[[tag2]]");

        let result = run(true, Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_json_succeeds_with_empty_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"");

        let result = run(true, Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }
}
