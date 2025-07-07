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

### For Each TODO (Repeat steps 3-5):

### 3. Implementation (Code Mode)
- Switch to Code mode for actual implementation
- Focus on completing one TODO at a time
- Write clean, maintainable code following Rust best practices
- Use Claude Code's TODO functionality to track progress

### 4. Quality Checks
Before committing any code, run the following checks in order (based on CI configuration):

Before each commit, ensure:
- [ ] Code builds successfully (`cargo build --verbose --workspace`)
- [ ] All tests pass (`cargo test --verbose --workspace`)
- [ ] Code is properly formatted (`cargo fmt --all -- --check`)
- [ ] No clippy warnings (`cargo clippy --all-targets --all-features`)

### 5. Commit
- Create a descriptive commit message for the completed TODO
- Follow conventional commit format when possible
- Example: `feat: implement search functionality for tags`