use clap_verbosity_flag::{log, Verbosity, WarnLevel};
use std::path::Path;
use std::time::Instant;
use tracing::{span, Level};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::output::build_renderer::BuildRendererImpl;
use crate::usecase::build::model::color_scheme::ColorScheme;
use crate::usecase::build::model::css::CssMetadata;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::list_view_configs::ListViewConfigs;
use crate::usecase::build::model::paging::Paging;
use crate::usecase::build::model::sort::SortKey;
use crate::usecase::build::usecase::BuildUsecase;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::usecase::progress::Progress;
use scraps_libs::git::GitCommandImpl;

pub fn run(
    verbose: Verbosity<WarnLevel>,
    git: bool,
    project_path: Option<&Path>,
) -> ScrapsResult<()> {
    let log_level = match verbose.log_level() {
        Some(log::Level::Error) => Level::ERROR,
        Some(log::Level::Warn) => Level::WARN,
        Some(log::Level::Info) => Level::INFO,
        Some(log::Level::Debug) => Level::DEBUG,
        Some(log::Level::Trace) => Level::TRACE,
        None => Level::WARN,
    };
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(log_level)
        .init();
    let span_run = span!(Level::INFO, "run").entered();
    let result = execute(git, project_path);
    span_run.exit();
    result
}

fn execute(git: bool, project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let ssg = config.require_ssg()?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let static_dir_path = path_resolver.static_dir();
    let output_dir_path = path_resolver.output_dir(&config);

    // Input: read scraps (with git timestamps if --git is set) and README
    let git_command = git.then(GitCommandImpl::new);
    let (scraps_with_ts, readme_text) =
        read_scraps::to_all_scraps_with_timestamps(&scraps_dir_path, git_command)?;

    let renderer = BuildRendererImpl::new(&static_dir_path, &output_dir_path);
    let usecase = BuildUsecase::new();
    let progress = ProgressImpl::init(Instant::now());
    let base_url = ssg.base_url();
    let title = &ssg.title;
    let default_lang_code = scraps_libs::lang::LangCode::default();
    let lang_code = ssg
        .lang_code
        .as_ref()
        .map(|c| c.as_lang_code())
        .unwrap_or(&default_lang_code);
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    let html_metadata = HtmlMetadata::new(lang_code, title, &ssg.description, &ssg.favicon);
    let default_color_scheme = ColorScheme::OsSetting;
    let css_metadata = CssMetadata::new(
        ssg.color_scheme
            .as_ref()
            .map(|c| c.as_color_scheme())
            .unwrap_or(&default_color_scheme),
    );
    let build_search_index = ssg.build_search_index.unwrap_or(true);
    let default_sort_key = SortKey::CommittedDate;
    let sort_key = ssg
        .sort_key
        .as_ref()
        .map(|s| s.as_sort_key())
        .unwrap_or(&default_sort_key);
    let paging = match ssg.paginate_by {
        None => Paging::Not,
        Some(u) => Paging::By(u),
    };
    let list_view_configs = ListViewConfigs::new(&build_search_index, sort_key, &paging);

    usecase.execute(
        &scraps_with_ts,
        &readme_text,
        &progress,
        &renderer,
        &base_url,
        timezone,
        &html_metadata,
        &css_metadata,
        &list_view_configs,
    )?;
    progress.end();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;
    use std::fs;

    #[rstest]
    fn run_generates_html_and_css(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"")
            .add_scrap("test1.md", b"# header1\n## header2\n")
            .add_scrap("test2.md", b"[[test1]]\n");

        let result = execute(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());

        // Verify scrap HTMLs generated
        let html1 = fs::read_to_string(project.output_path("scraps/test1.html")).unwrap();
        assert!(!html1.is_empty());
        let html2 = fs::read_to_string(project.output_path("scraps/test2.html")).unwrap();
        assert!(!html2.is_empty());

        // Verify index.html generated
        let index = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert!(!index.is_empty());

        // Verify CSS generated
        let css = fs::read_to_string(project.output_path("main.css")).unwrap();
        assert!(!css.is_empty());

        // Verify search index JSON generated (default: true)
        let json = fs::read_to_string(project.output_path("search_index.json")).unwrap();
        assert!(!json.is_empty());
    }

    #[rstest]
    fn run_skips_search_index_when_disabled(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(
                b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"\nbuild_search_index = false",
            )
            .add_scrap("test1.md", b"# header1\n")
            .add_scrap("test2.md", b"[[test1]]\n");

        let result = execute(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());

        // Verify search index JSON not generated
        let json = fs::read_to_string(project.output_path("search_index.json"));
        assert!(json.is_err());
    }

    #[rstest]
    fn run_renders_tag_links_and_embeds(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"")
            .add_scrap(
                "source.md",
                b"Tagged #[[ai/ml]]\n\nEmbed: ![[target#Notes]]",
            )
            .add_scrap(
                "target.md",
                b"# Target\n\n## Notes\n\nembedded body\n\n## Other\n\nhidden body\n",
            );

        let result = execute(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());

        let html = fs::read_to_string(project.output_path("scraps/source.html")).unwrap();
        assert!(html.contains("http://localhost:1112/tags/ai/ml.html"));
        assert!(html.contains("embedded body"));
        assert!(!html.contains("hidden body"));
    }

    #[rstest]
    fn run_generates_tag_links_with_tags_path(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"")
            .add_scrap("source.md", b"#[[ai]]");

        let result = execute(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());

        let index = fs::read_to_string(project.output_path("index.html")).unwrap();
        assert!(index.contains("tags/ai.html"));
    }

    #[rstest]
    fn run_with_git_flag_includes_commited_date_block(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"")
            .add_scrap("test1.md", b"# header1\n");

        let result = execute(true, Some(project.project_root.as_path()));
        assert!(result.is_ok());

        // Outside a git repo `commited_ts` is None so the conditional block
        // is omitted; this test just asserts that --git does not error out.
        let html = fs::read_to_string(project.output_path("scraps/test1.html")).unwrap();
        assert!(!html.is_empty());
    }
}
