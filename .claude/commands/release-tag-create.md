---
description: Create a release tag with version bump
allowed-tools: ["Bash(git branch:*)", "Bash(git stash:*)", "Bash(git checkout:*)", "Bash(git fetch:*)", "Bash(git pull:*)", "Bash(git status:*)", "Bash(git log:*)", "Bash(git add:*)", "Bash(git commit:*)", "Bash(git push:*)", "Bash(git tag:*)", "Bash(git merge-base:*)", "Bash(mise run cargo:build)", "Bash(gh pr create:*)", "Bash(gh pr checks:*)", "Bash(gh pr view:*)", "Read", "Edit"]
---

Create a release tag for the Scraps project using the following workflow:

**Arguments**:
- `$ARGUMENTS`: Version number (e.g., "0.27.0")

**Workflow**:

1. **Create a release branch from up-to-date main**:
   - Check current branch with `git branch --show-current`
   - If not on main, stash changes with `git stash` and checkout main with `git checkout main`
   - Pull latest changes with `git pull`
   - Create the release branch: `git checkout -b release/v$ARGUMENTS`
   - Never commit the version bump on main: main rejects direct pushes

2. **Update version in Cargo.toml files**:
   - Update `version = "X.Y.Z"` in `/Cargo.toml` (two occurrences: `[workspace.package]` and `[workspace.dependencies.scraps_libs]`)
   - Update `version = "X.Y.Z"` in `/modules/libs/Cargo.toml`
   - Run `mise run cargo:build` so `Cargo.lock` picks up both `scraps` and `scraps_libs`

3. **Commit version bump and push the branch**:
   - Add all changed files: `Cargo.toml`, `Cargo.lock`, `modules/libs/Cargo.toml`
   - Commit with message format: `v$ARGUMENTS` (e.g., "v0.27.0")
   - Include Claude Code attribution in commit body
   - Push the branch: `git push -u origin release/v$ARGUMENTS`

4. **Open the release PR**:
   - `gh pr create --base main --title "v$ARGUMENTS" --body "<summary>"`
   - Show the PR URL to the user

5. **Wait for required checks and merge**:
   - Watch required checks with `gh pr checks --watch` (`build` and `zizmor`)
   - main also requires an approving review, and the agent cannot approve or merge its own PR: ask the user to merge
   - Do not continue until `gh pr view --json state,mergedAt` reports the PR as `MERGED`

6. **Tag the merged commit** (only after the merge is confirmed):
   - `git checkout main && git pull`
   - Verify `version` in `Cargo.toml` matches `$ARGUMENTS` and `git log -1 --oneline` is the merged version bump
   - Create tag: `git tag v$ARGUMENTS`
   - Request confirmation before pushing tag
   - Push tag: `git push origin v$ARGUMENTS` (requires user approval)

7. **Verify**:
   - Confirm tag creation with `git tag --sort=-v:refname | head -5`
   - Confirm the tag is on main with `git merge-base --is-ancestor v$ARGUMENTS origin/main`

**Usage**: `/release-tag-create 0.27.0`

**Example**:
```bash
# For version 0.27.0
/release-tag-create 0.27.0
```

**Notes**:
- The version bump must go through a PR: main blocks direct pushes and requires the `build` and `zizmor` checks plus a review approval
- Never create or push the tag before the PR is merged; a tag that is not an ancestor of main has to be deleted with `git push origin :refs/tags/v$ARGUMENTS` and recreated
- The version format should be semver without 'v' prefix in arguments
- Tag will be created with 'v' prefix (e.g., v0.27.0)
- User confirmation is required before pushing the tag: publishing a GitHub Release from it triggers the crates.io publish, the homebrew-tap update, and the floating v{major}/v{major}.{minor} tag moves (see `.github/workflows/release.yml`)
