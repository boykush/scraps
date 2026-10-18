use clap_verbosity_flag::{Verbosity, WarnLevel, log};
use std::path::Path;
use std::time::Instant;
use tracing::{Level, span};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::output::build_renderer::BuildRendererImpl;
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
    let build_config = config.to_build_config(None)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let static_dir_path = path_resolver.static_dir();
    let output_dir_path = path_resolver.output_dir(&config);

    // Input: read scraps (with git timestamps unless --no-git) and README.
    // The wiki root is the project root, so skip `static/` and the configured
    // output directory at the top level.
    let git_command = git.then(GitCommandImpl::new);
    let exclude_dirs = vec![static_dir_path.clone(), output_dir_path.clone()];
    let (scraps_with_ts, readme_text) =
        loader::load_scraps_with_timestamps(&scraps_dir_path, &exclude_dirs, git_command)?;

    let renderer = BuildRendererImpl::new(&static_dir_path, &output_dir_path)?;
    let usecase = BuildUsecase::new();
    let progress = ProgressImpl::init(Instant::now());

    usecase.execute(
        &scraps_with_ts,
        &readme_text,
        &progress,
        &renderer,
        &build_config,
    )?;
    progress.end();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
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
        assert!(html.contains("http://localhost:1112/tags/ai/ml/"));
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
        assert!(index.contains("tags/ai/"));
    }

    #[rstest]
    fn run_writes_every_name_to_a_page_of_its_own(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"")
            .add_scrap("a b.md", b"spaced #[[a b]] #[[index]]")
            .add_scrap("a-b.md", b"dashed #[[a-b]]")
            .add_scrap("What is it?.md", b"asked")
            .add_scrap("What is it question.md", b"spelled out")
            .add_scrap("index.md", b"a scrap named index")
            .add_scrap(
                "hub.md",
                b"[[a b]] [[a-b]] [[What is it?]] [dashed](a-b.md)",
            )
            .add_scrap("typed.md", b"---\ntype: index\n---\n");

        let result = execute(false, Some(project.project_root.as_path()));
        assert!(result.is_ok());

        let read = |path: &str| fs::read_to_string(project.output_path(path)).unwrap();

        // Names that used to slugify to one path each keep their own page.
        assert!(read("scraps/a b.html").contains("spaced"));
        assert!(read("scraps/a-b.html").contains("dashed"));
        assert!(read("scraps/What is it?.html").contains("asked"));
        assert!(read("scraps/What is it question.html").contains("spelled out"));
        assert!(read("tags/a b/index.html").contains("scraps/a%20b.html"));
        assert!(read("tags/a-b/index.html").contains("scraps/a-b.html"));

        // A scrap and a tag named `index` are pages like any other, and a
        // type named `index` is a query on a list rather than a path.
        assert!(read("scraps/index.html").contains("a scrap named index"));
        assert!(read("tags/index/index.html").contains("1 of 7 scraps"));
        assert!(read("index.html").contains(r#"href="?type=index""#));
        assert!(read("all-tags/index.html").contains("class=\"tag-links\""));
        assert!(read("index.html").contains("all scraps"));

        let hub = read("scraps/hub.html");
        for href in [
            "http://localhost:1112/scraps/a%20b.html",
            "http://localhost:1112/scraps/a-b.html",
            "http://localhost:1112/scraps/What%20is%20it%3F.html",
        ] {
            assert!(hub.contains(&format!("href=\"{href}\"")), "missing {href}");
        }
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
        // is omitted; this test just asserts that the git read does not error out.
        let html = fs::read_to_string(project.output_path("scraps/test1.html")).unwrap();
        assert!(!html.is_empty());
    }
}
