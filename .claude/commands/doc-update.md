---
description: Propose documentation updates based on GitHub Release
allowed-tools: ["Bash(gh release view:*)", "Read", "Grep", "Glob"]
---

Analyze a GitHub Release and propose documentation updates for the Scraps project:

**Arguments**:
- `$ARGUMENTS`: Version number (e.g., "0.27.0")

**Workflow**:

1. **Fetch release information**:
   - Get release details: `gh release view v$ARGUMENTS --json body,name --jq '{name: .name, body: .body}'`
   - Extract features and significant changes from release notes

2. **Analyze current documentation**:
   - Read `/README.md` to understand current feature list and description
   - Search scraps directory for relevant documentation files:
     - `scraps/What is Scraps?.md` - Core concept and capabilities
     - `scraps/Getting Started.md` - Getting started guide
     - `scraps/Configuration.md` - Configuration documentation
     - `scraps/Feature/*.md` - Feature-specific documentation
     - `scraps/CLI/*.md` - CLI command documentation
   - Use `Grep` and `Read` to identify sections that may need updates

3. **Generate update proposals**:
   For each feature or significant change in the release:
   - **Identify affected files**: Determine which documentation files should be updated
   - **Propose specific changes**: For each file, suggest:
     - What content to add/modify
     - Where to place it (specific section or line)
     - Why this update is needed

   Common update patterns:
   - **New features**:
     - Add to `/README.md` Features section if user-facing
     - Create or update relevant `scraps/Feature/*.md` documentation
     - Update `scraps/Configuration.md` if new config options added
     - Update relevant `scraps/CLI/*.md` if CLI commands affected
   - **Configuration changes**:
     - Update `scraps/Configuration.md` with new options and examples
     - Update `scraps/Getting Started.md` if it affects initial setup

4. **Present proposals to user**:
   - Show proposed changes grouped by file
   - Explain the rationale for each update based on release notes
   - Ask user which updates to proceed with

5. **User review and selection**:
   - User can approve all, select specific updates, or request modifications
   - User can provide additional context or requirements for updates

**Usage**: `/doc-update 0.27.0`

**Example**:
```bash
# After creating release v0.27.0
/doc-update 0.27.0
```

**Notes**:
- This command only proposes changes, it does not automatically update files
- User must review and approve updates before implementation
- Updates should maintain consistency with existing documentation style
- Focus on user-facing changes that affect how users interact with Scraps
