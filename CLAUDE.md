# Scraps Development with Claude Code

For comprehensive guidelines, see @CONTRIBUTING.md

## Project Structure

- Workspace: root crate (`src/`) + `modules/libs/` (scraps_libs)
- CLI commands: `src/cli/cmd/<name>.rs` using `PathResolver` + `ScrapConfig`
- Usecases: `src/usecase/<name>/usecase.rs`
- Libs features: `error`, `git`, `lang`, `markdown`, `model`, `search` — gated in `modules/libs/src/lib.rs`
- Config language: PKL (`*.pkl` files)
- Discovery: `livt/` is the product discovery workspace ([livt](https://github.com/boykush/livt)); its content is written in Japanese by convention, while the rest of the repository is English

## Development Workflow

1. **Plan**: Start in Plan mode to analyze requirements and design approach
2. **Implement**: One TODO at a time, following TDD (Red -> Green -> Refactor)
   - PostToolUse hook auto-formats `.rs` files on Edit/Write
   - Pre-commit hook runs `cargo:quality` (test + fmt + clippy) automatically
3. **Commit & PR**: Use `/commit` or `/commit-push-pr` skill (see @CONTRIBUTING.md)
