# Scraps - Development Guide

Welcome to the Scraps development community! 🎉

Scraps is a static site generator that brings developer-friendly workflows to documentation, using Markdown files with simple Wiki-link notation.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Quick Links

- **Official Documentation:** https://boykush.github.io/scraps/
- **Repository:** https://github.com/boykush/scraps
- **Deep Wiki:** https://deepwiki.com/boykush/scraps
- **Sample Site (Japanese):** https://boykush.github.io/wiki/
- **Crates.io:** https://crates.io/crates/scraps

## Ways to Contribute

There are many ways you can help improve Scraps:

### 🐛 Report Bugs
Found a bug? Please use our [Bug Report Template](https://github.com/boykush/scraps/issues/new?assignees=&labels=bug&projects=&template=bug-report-template.md&title=) to report it. Make sure to:
- Add a `context:` label to help us categorize the issue
- Include your Scraps version (`scraps -V`)
- Provide clear reproduction steps

### 💡 Suggest Features
Have an idea for a new feature? We'd love to hear it! Use our [Enhancement Feature Template](https://github.com/boykush/scraps/issues/new?assignees=&labels=enhancement&projects=&template=enhancement-feature-template.md&title=) to:
- Add a `context:` label for proper categorization
- Describe your idea and requirements clearly
- Explain how it would benefit the community

### 📖 Improve Documentation
Help us make Scraps more accessible by:
- Improving existing documentation in the `docs/` directory
- Fixing typos or unclear explanations

---

## Claude Code Plugins

This project uses [Claude Code](https://claude.com/claude-code) plugins for development workflow:

- `/commit` — Create a conventional commit
- `/commit-push-pr` — Create branch, commit, push, and open a PR in one step

---

## Testing Guidelines

### Overview

Scraps maintains a comprehensive testing strategy:
- **Small Tests**: Fast tests for individual functions and methods
- **Medium Tests**: Integration tests using tempfile + rstest fixtures for file
  system operations
- **Performance Tests**: Automated build time validation (≤ 3 seconds)

Browser-runtime checks (search box, OGP card, CDN script loading) are
verified manually on the deployed docs site after each release. The
in-source HTML output is kept honest by Rust render tests asserting
template-level invariants (e.g. fuse.js loads as an ES module).

## Development Environment Setup

The easiest way to set up your development environment is using [mise](https://mise.jdx.dev/):

```bash
# Navigate to the project directory
cd scraps

# Install all required tools and dependencies
mise install
```

This will automatically install the correct versions of:
- **Rust** (stable version)
- **hk** (git hook manager)
- **pkl** (configuration language)
- Any other tools specified in the project configuration

### Git Pre-commit Hooks

This project uses [hk](https://hk.jdx.dev/) for managing git hooks. Pre-commit hooks are automatically installed when you run `mise install`.

The pre-commit hook runs quality checks on staged files:
- **Rust files (*.rs)**: Runs `mise run cargo:quality` which includes:
  - Build verification
  - All tests
  - Code formatting check (rustfmt)
  - Linter checks (clippy)
- **PKL files (*.pkl)**: Validates configuration syntax

If any check fails, the commit will be blocked. Fix the issues and try again.

To manually run the pre-commit checks:
```bash
mise exec -- hk run pre-commit
```

---

### Prerequisites for Testing

Before running tests, ensure you have the development environment set up as described above.

### Small Tests and Medium Tests

#### Running Tests

```bash
# Run all workspace tests
mise run cargo:test
```

#### Writing Medium Tests

Scraps uses tempfile + rstest fixtures for integration tests. The fixtures are
defined in `src/test_fixtures.rs` and provide automatic cleanup:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::TempScrapProject;

    #[test]
    fn test_functionality() {
        let project = TempScrapProject::new();

        // Setup test files using builder pattern
        project
            .add_scrap("test.md", b"# Test Content")
            .add_static_file("index.html", b"<html></html>");

        // Your test logic here
        assert_eq!(expected, actual);

        // Automatic cleanup when project goes out of scope
    }
}
```

**Available Fixtures:**

- `TempScrapProject`: Full project structure (scraps_dir, static_dir,
  public_dir, templates_dir)
- `SimpleTempDir`: Single temporary directory for simple tests

### Performance Tests

#### Build Time Requirements ⚡

**Critical Requirement**: All changes must maintain build times ≤ 3 seconds.

The performance test runs automatically on every pull request and:
- Builds Scraps in release mode (`cargo build --release`)
- Tests against the [boykush/wiki](https://github.com/boykush/wiki) repository
- Measures `scraps build -v` execution time
- **Fails the PR if build time exceeds 3 seconds**