use std::io::Write;
use std::path::Path;

use itertools::Itertools;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::display::tag::{DisplayTag, DisplayTagTable};
use crate::cli::json::tag::TagJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::tag::list::usecase::ListTagUsecase;

pub fn run(json: bool, project_path: Option<&Path>, writer: &mut impl Write) -> ScrapsResult<()> {
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
                let backlinks_count = backlinks_map.get_tag(&tag).len();
                TagJson {
                    title: tag.to_string(),
                    backlinks_count,
                }
            })
            .sorted_by(|a, b| b.backlinks_count.cmp(&a.backlinks_count))
            .collect();
        writeln!(writer, "{}", serde_json::to_string(&tags_json)?)?;
    } else {
        let base_url = config.get_base_url();
        let display_tags = tags
            .into_iter()
            .map(|tag| DisplayTag::new(&tag, base_url.as_ref(), &backlinks_map))
            .collect::<ScrapsResult<Vec<DisplayTag>>>()?;

        let sorted = display_tags
            .into_iter()
            .sorted_by_key(|tag| tag.backlinks_count())
            .rev()
            .collect::<Vec<_>>();
        let table = DisplayTagTable::new(sorted);
        writeln!(writer, "{table}")?;
    }
    Ok(())
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

        let mut buf = Vec::new();
        let result = run(false, Some(project.project_root.as_path()), &mut buf);
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_without_ssg_section(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("a.md", b"#[[tag1]]");

        let mut buf = Vec::new();
        let result = run(false, Some(project.project_root.as_path()), &mut buf);
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_with_empty_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"");

        let mut buf = Vec::new();
        let result = run(false, Some(project.project_root.as_path()), &mut buf);
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(false, Some(project.project_root.as_path()), &mut buf);
        assert!(result.is_err());
    }

    #[rstest]
    fn run_json_outputs_sorted_tags(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"#[[tag1]]")
            .add_scrap("b.md", b"#[[tag1]] #[[tag2]]");

        let mut buf = Vec::new();
        run(true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let tags: Vec<TagJson> = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].title, "tag1");
        assert_eq!(tags[0].backlinks_count, 2);
        assert_eq!(tags[1].title, "tag2");
        assert_eq!(tags[1].backlinks_count, 1);
    }

    #[rstest]
    fn run_json_outputs_empty_array(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"");

        let mut buf = Vec::new();
        run(true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let tags: Vec<TagJson> = serde_json::from_str(output.trim()).unwrap();
        assert!(tags.is_empty());
    }
}
