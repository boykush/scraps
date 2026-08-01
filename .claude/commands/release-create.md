---
description: Create a GitHub Release for an existing git tag
allowed-tools: ["Bash(git tag:*)", "Bash(git log:*)", "Bash(git show:*)", "Bash(git fetch:*)", "Bash(git merge-base:*)", "Bash(gh pr view:*)", "Bash(gh api:*)", "Bash(gh release create:*)", "Bash(gh release view:*)", "Bash(gh run list:*)", "Bash(gh run view:*)", "Bash(gh run watch:*)", "Read", "Grep"]
---

Create a GitHub Release for the Scraps project using the following workflow:

**Arguments**:
- `$ARGUMENTS`: Version number (e.g., "0.27.0")

**Workflow**:

1. **Verify tag exists and points at main**:
   - Check if tag `v$ARGUMENTS` exists with `git tag --list v$ARGUMENTS`
   - If tag doesn't exist, inform user to run `/release-tag-create` first
   - Check the tag is an ancestor of main: `git fetch origin main` then `git merge-base --is-ancestor v$ARGUMENTS origin/main`
   - If it isn't, stop: the tag must be deleted (`git push origin :refs/tags/v$ARGUMENTS`) and recreated on main via `/release-tag-create`

2. **Find previous version tag**:
   - Get sorted list of tags: `git tag --sort=-v:refname`
   - Identify the previous version tag (the one before `v$ARGUMENTS`)

3. **Analyze commit history and PRs**:
   - Get commits between previous tag and current tag: `git log <previous_tag>..v$ARGUMENTS --oneline`
   - For each merge commit, extract PR number (e.g., from "Merge pull request #123")
   - Fetch PR details using `gh pr view <PR_number>` or `gh api repos/boykush/scraps/pulls/<PR_number>`
   - Extract PR title, labels, and categorize by:
     - Labels or PR title prefix analysis:
       - `bug`, `fix:` → 🐛 Bug Fixes
       - `enhancement`, `feat:`, `feature` → ✨ Features
       - `documentation`, `docs:` → 📚 Documentation
       - `refactoring`, `refactor:` → 🔧 Refactoring
       - `test`, `tests` → ✅ Tests
       - `maintenance`, `chore:` → 🔧 Maintenance
     - Default → 🔧 Other Changes

4. **Generate release notes**:
   - Create markdown format with categorized changes
   - Include PR numbers and titles from PR data
   - **For Features section**: Add a brief description (2-3 lines) explaining what the feature does and its benefits
   - Add "Full Changelog" link: `https://github.com/boykush/scraps/compare/<previous_tag>...v$ARGUMENTS`
   - Format example:
     ```markdown
     ## What's Changed

     ### ✨ Features
     - feat: add configurable scraps directory support (#123)

       Users can now customize the scraps directory location via .scraps.toml.
       This allows for better project organization and flexibility in documentation structure.

     ### 🐛 Bug Fixes
     - fix: update E2E test to match new project description (#182)

     ### 🔧 Maintenance
     - chore: update rmcp dependency to v0.6.3 (#181)

     **Full Changelog**: https://github.com/boykush/scraps/compare/v0.26.1...v0.27.0
     ```

5. **Request user input for feature descriptions**:
   - For each feature item, present the PR title and ask user to provide a brief description
   - User can provide description or skip if not needed

6. **Request confirmation**:
   - Show generated release notes to user
   - Ask for confirmation before creating the release

7. **Create GitHub Release** (requires user approval):
   - Use `gh release create v$ARGUMENTS --title "v$ARGUMENTS" --notes "<generated_notes>"`
   - Mark as latest release

8. **Verify**:
   - Confirm release creation with `gh release view v$ARGUMENTS`
   - Display release URL
   - Follow the triggered workflow: `gh run list --workflow=release.yml --limit 1`, then `gh run watch <run_id> --exit-status`
   - Report each job: `publish-crate`, `update-homebrew`, and `update-floating-tags` run after `build` and can fail independently, leaving the release half-published

**Usage**: `/release-create 0.27.0`

**Example**:
```bash
# For version 0.27.0 (after running /release-tag-create 0.27.0)
/release-create 0.27.0
```

**Notes**:
- This command requires that the git tag already exists (created by `/release-tag-create`) and is an ancestor of main
- Creating the Release triggers the crates.io publish, the homebrew-tap update, and the floating v{major}/v{major}.{minor} tag moves (see `.github/workflows/release.yml`); a tag off main would ship a build that is not on main
- The version format should be semver without 'v' prefix in arguments
- Release notes are automatically generated from PR information
- User confirmation is required before creating the GitHub Release
- Categorization is based on PR labels and titles
