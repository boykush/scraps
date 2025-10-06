---
description: Create a release tag with version bump
allowed-tools: ["Bash(git branch:*)", "Bash(git stash:*)", "Bash(git checkout:*)", "Bash(git pull:*)", "Bash(git status:*)", "Bash(git add:*)", "Bash(git commit:*)", "Bash(git tag:*)", "Read", "Edit"]
---

Create a release tag for the Scraps project using the following workflow:

**Arguments**:
- `$ARGUMENTS`: Version number (e.g., "0.27.0")

**Workflow**:

1. **Verify current branch is main**:
   - Check current branch with `git branch --show-current`
   - If not on main, stash changes with `git stash` and checkout main with `git checkout main`
   - Pull latest changes with `git pull`

2. **Update version in Cargo.toml files**:
   - Update `version = "X.Y.Z"` in `/Cargo.toml` (replace all occurrences)
   - Update `version = "X.Y.Z"` in `/modules/libs/Cargo.toml`

3. **Commit version bump**:
   - Add all changed files: `Cargo.toml`, `Cargo.lock`, `modules/libs/Cargo.toml`
   - Commit with message format: `v$ARGUMENTS` (e.g., "v0.27.0")
   - Include Claude Code attribution in commit body

4. **Request confirmation before push**:
   - Show the commit details
   - Ask user to confirm before proceeding with push

5. **Push commit to remote** (requires user approval):
   - Push to origin main: `git push`

6. **Create and push git tag**:
   - Create tag: `git tag v$ARGUMENTS`
   - Request confirmation before pushing tag
   - Push tag: `git push origin v$ARGUMENTS` (requires user approval)

7. **Verify**:
   - Confirm tag creation with `git tag --sort=-v:refname | head -5`

**Usage**: `/release-tag-create 0.27.0`

**Example**:
```bash
# For version 0.27.0
/release-tag-create 0.27.0
```

**Notes**:
- This command should only be run from main branch
- Ensure all changes for the release are already merged to main
- The version format should be semver without 'v' prefix in arguments
- Tag will be created with 'v' prefix (e.g., v0.27.0)
- User confirmation is required before pushing commits and tags
