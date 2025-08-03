---
description: Create a pull request with auto-generated title and description
allowed-tools: ["Bash", "Grep", "Read"]
---

Create a pull request for the current branch using the following workflow:

1. **Analyze current branch and changes**:
   - Check git status and commit history since main branch
   - Ensure we're not on main branch

2. **Generate PR content**:
   - Create descriptive title based on commits and branch name
   - Follow the format specified in @.github/pull_request_template.md

3. **Create the pull request**:
   - Use `gh pr create` with generated content and base branch main
   - Handle additional arguments: $ARGUMENTS

4. **Verify and report**:
   - Display PR URL and confirmation

**Usage**: `/pr-create [--draft] [--base=branch]`