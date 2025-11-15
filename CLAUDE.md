# Scraps Development with Claude Code

This document outlines the development workflow for contributing to the Scraps project using Claude Code.

For comprehensive development guidelines, testing procedures, and contribution requirements, see @CONTRIBUTING.md

## 🚀 Claude Code Development Workflow

### 1. Planning Phase (Plan Mode)

- Start in Plan mode to analyze requirements and create implementation plans
- Break down the task into manageable TODOs
- Design the overall approach and identify potential challenges
- Create a clear roadmap for implementation

### 2. Implementation Setup (Branch Creation)

- **IMPORTANT**: Never work directly on the main branch. Always create a feature branch first.
- Create a new feature branch with descriptive naming:

  ```bash
  git checkout -b feature/your-feature-name
  ```

- Ensure you're working from the latest main branch

### For Each TODO (Repeat steps 3-5)

### 3. Implementation (Code Mode)

- Switch to Code mode for actual implementation
- Focus on completing one TODO at a time
- Write clean, maintainable code following Rust best practices
- Use Claude Code's TODO functionality to track progress
- **TDD Approach**: For new features, follow Test-Driven Development:
  1. **Red Phase**: Write failing tests first
  2. **Green Phase**: Implement minimal code to make tests pass
  3. **Refactor Phase**: Improve code quality while keeping tests green
  4. Commit after each phase with descriptive messages

### 4. Quality Checks

Before committing any code, run the following checks in order (based on CI configuration):

You can run all quality checks at once:

```bash
mise run cargo:quality
```

Or run individual checks:

- [ ] Code builds successfully: `mise run cargo:build`
- [ ] All tests pass: `mise run cargo:test`
- [ ] Code is properly formatted: `mise run cargo:fmt`
- [ ] No clippy warnings: `mise run cargo:clippy`

To automatically fix formatting issues:

```bash
mise run cargo:fmt-fix
```

**Note**: Pre-commit hooks are automatically set up via hk and will run these quality checks when you commit. If any check fails, the commit will be blocked.

### 5. Commit

- Create a descriptive commit message for the completed TODO
- Follow conventional commit format when possible
- Example: `feat: implement search functionality for tags`
