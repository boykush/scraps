use std::fs;
use std::path::Path;

use tera::{Context, Tera};

use crate::error::{BuildError, ScrapsResult, anyhow::Context as _};

/// Render `template_name` into `file_path`, creating parent directories first
/// so callers can write into nested context/tag paths.
///
/// Rendered in memory first so each page lands in one write: tera emits many
/// small writes, and most pages outgrow a default-sized write buffer.
pub fn render_to_file(
    tera: &Tera,
    template_name: &str,
    context: &Context,
    file_path: &Path,
) -> ScrapsResult<()> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).context(BuildError::CreateDir)?;
    }
    let write_failure = || BuildError::WriteFailure(file_path.to_path_buf());

    let html = tera
        .render(template_name, context)
        .context(write_failure())?;
    fs::write(file_path, html).context(write_failure())
}

/// The `static/` glob tera loads user overrides from. A path that is not valid
/// UTF-8 is lossy-converted, so it matches nothing instead of failing the build.
pub fn user_template_glob(static_dir_path: &Path, pattern: &str) -> String {
    static_dir_path.join(pattern).to_string_lossy().into_owned()
}

/// A same-named template in the project's `static/` directory overrides the
/// bundled one, so prefer `user` whenever the glob loaded it.
pub fn resolve_template<'a>(tera: &Tera, user: &'a str, builtin: &'a str) -> &'a str {
    if tera.get_template_names().any(|t| t == user) {
        user
    } else {
        builtin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rstest::rstest;

    fn tera_with(templates: Vec<(&str, &str)>) -> Tera {
        let mut tera = Tera::default();
        tera.add_raw_templates(templates).unwrap();
        tera
    }

    #[rstest]
    fn render_to_file_creates_missing_parent_dirs(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let tera = tera_with(vec![("page.html", "hello {{ name }}")]);
        let mut context = Context::new();
        context.insert("name", "scraps");
        let file_path = project.output_dir.join("nested/deep/page.html");

        render_to_file(&tera, "page.html", &context, &file_path).unwrap();

        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello scraps");
    }

    #[rstest]
    fn render_to_file_reports_the_path_on_failure(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let tera = tera_with(vec![("page.html", "{{ missing }}")]);
        let file_path = project.output_dir.join("page.html");

        let result = render_to_file(&tera, "page.html", &Context::new(), &file_path);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains(&file_path.to_string_lossy().to_string())
        );
    }

    #[rstest]
    fn resolve_template_prefers_the_user_template() {
        let tera = tera_with(vec![
            ("index.html", "user"),
            ("__builtins/index.html", "builtin"),
        ]);

        assert_eq!(
            resolve_template(&tera, "index.html", "__builtins/index.html"),
            "index.html"
        );
    }

    #[rstest]
    fn resolve_template_falls_back_to_the_builtin() {
        let tera = tera_with(vec![("__builtins/index.html", "builtin")]);

        assert_eq!(
            resolve_template(&tera, "index.html", "__builtins/index.html"),
            "__builtins/index.html"
        );
    }
}
