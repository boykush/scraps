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

## Testing Guidelines

### Overview

Scraps maintains a comprehensive testing strategy with three main types of tests:
- **Small Tests**: Fast tests for individual functions and methods
- **Medium Tests**: Integration tests using TestResources for file system operations
- **E2E Tests**: Large, browser-based end-to-end tests using Playwright
- **Performance Tests**: Automated build time validation (≤ 3 seconds)

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
- **Node.js** (for E2E tests)
- Any other tools specified in the project configuration

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

Scraps uses a custom test helper system located in `modules/libs/src/tests.rs`. Here's how to write effective unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::tests::TestResources;
    use std::path::PathBuf;

    #[test]
    fn test_functionality() {
        let mut resources = TestResources::new();
        
        // Setup test files and directories
        resources
            .add_file(&PathBuf::from("test.md"), b"# Test Content")
            .add_dir(&PathBuf::from("output"));
        
        // Run test with automatic cleanup
        resources.run(|| {
            // Your test logic here
            assert_eq!(expected, actual);
        });
    }
}
```

### E2E Tests

#### Running E2E Tests

```bash
# Run all E2E tests
mise run e2e
```

#### E2E Test Configuration

E2E tests are configured to:
- Use three browsers: Chromium, Firefox, and WebKit
- Automatically start `cargo run serve` on `http://127.0.0.1:1112`
- Generate HTML reports for test results

#### Writing E2E Tests

```typescript
import { test, expect } from '@playwright/test';
  // Navigate to page
  await page.goto('/your-page');
  
  // Test interactions
  await page.locator('#element-id').click();
  await expect(page.locator('#result')).toBeVisible();
});
```

### Performance Tests

#### Build Time Requirements ⚡

**Critical Requirement**: All changes must maintain build times ≤ 3 seconds.

The performance test runs automatically on every pull request and:
- Builds Scraps in release mode (`cargo build --release`)
- Tests against the [boykush/wiki](https://github.com/boykush/wiki) repository
- Measures `scraps build -v` execution time
- **Fails the PR if build time exceeds 3 seconds**