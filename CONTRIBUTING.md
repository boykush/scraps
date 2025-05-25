# Contributing to Scraps

Thank you for your interest in contributing to Scraps! This document provides guidelines and information for contributors.

## Testing Guidelines

### Overview

Scraps maintains a comprehensive testing strategy with three main types of tests:
- **Small Tests**: Fast tests for individual functions and methods
- **Medium Tests**: Integration tests using TestResources for file system operations
- **E2E Tests**: Large, browser-based end-to-end tests using Playwright
- **Performance Tests**: Automated build time validation (≤ 2.5 seconds)

### Prerequisites

Before running tests, ensure you have the following installed:

- **Rust** (latest stable version)
- **Node.js** (for E2E tests)
- **Git** (for cloning test repositories)

### Small Tests and Medium Tests

#### Running Tests

```bash
# Run all workspace tests
cargo test --workspace
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

#### Setup

```bash
# Navigate to E2E test directory
cd tests/e2e

# Install dependencies
npm install
```

#### Running E2E Tests

```bash
# Run all E2E tests
npx playwright test
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

**Critical Requirement**: All changes must maintain build times ≤ 2.5 seconds.

The performance test runs automatically on every pull request and:
- Builds Scraps in release mode (`cargo build --release`)
- Tests against the [boykush/wiki](https://github.com/boykush/wiki) repository
- Measures `scraps build -v` execution time
- **Fails the PR if build time exceeds 2.5 seconds**

#### Performance Test Workflow

```yaml
# Automatically triggered on PR
# Tests against real-world repository (boykush/wiki)
# Reports results as PR comment
```