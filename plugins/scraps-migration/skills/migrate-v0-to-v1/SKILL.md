---
name: migrate-v0-to-v1
description: Migrate a Scraps wiki from v0 to v1. Use this whenever the user asks to upgrade Scraps, migrate v0 wiki syntax/config, convert implicit tags to explicit `#[[tag]]`, move from `scraps-writer` to `scraps`, or audit a wiki for v1 readiness. This skill is designed for one-pass LLM-led migrations that discover the latest v0 and latest v1-or-newer CLI versions and run version-pinned audits.
allowed-tools: Read, Write, Edit, MultiEdit, Bash, Glob, Grep
user-invocable: true
argument-hint: [wiki-root]
---

# Migrate v0 to v1

Upgrade a Scraps v0 wiki to the v1 source shape with controlled edits and CLI
verification.

This is the migration tooling: v1 deliberately does not add a built-in
`scraps migrate` command. The LLM runs the old and new CLIs, builds an
inventory, edits the wiki, and reports anything that needs human judgment.

## Inputs

- `wiki-root`: directory containing the wiki. If omitted, use the current
  directory.

Do not ask the user for Scraps versions up front. Discover them before editing.

## CLI Runner

Resolve CLI versions in this order:

1. Refresh or inspect repository tags when the Scraps source repository is
   available:
   - `git fetch --tags` when network is allowed
   - `git tag --list 'v0.*' --sort=-version:refname | head -1`
   - `git tag --list 'v[1-9]*' --sort=-version:refname | head -1`
2. If tags are unavailable, inspect the wiki/repo for existing pins in
   `mise.toml`, GitHub Actions, lockfiles, README commands, or setup scripts.
3. If v0 still cannot be resolved, ask for the old version before making
   migration edits. v0 `tag list --json` is required for safe tag conversion.
4. If the v1-or-newer CLI cannot be resolved from tags, use the moving `v1`
   ref for GitHub Actions and the latest v1-compatible CLI available to `mise`.

At the time this skill was written, the latest known tags were:

- v0: `v0.33.0`
- v1-or-newer: `v1.0.0-rc.2`

Prefer dynamically resolving tags over hard-coding these examples.

Use these command forms when `mise` is available:

```bash
mise exec "github:boykush/scraps@$SCRAPS_V0_REF" -- scraps -C "$WIKI_ROOT" tag list --json
mise exec "github:boykush/scraps@$SCRAPS_V1_OR_NEWER_REF" -- scraps -C "$WIKI_ROOT" lint
```

If no version-pinned execution is available, use the `scraps` on PATH only after
checking `scraps --version` and confirming it matches the phase being run.

For v0 commands that predate `-C`, use the equivalent v0 directory flag if
needed. Prefer running from the wiki root over guessing when a v0 CLI does not
support `-C`.

## Migration Map

| v0 shape | v1 shape | How to migrate |
| --- | --- | --- |
| implicit tag fallback from unresolved `[[tag]]` | explicit `#[[tag]]` | Use v0 `tag list --json` as the source of truth; convert only links whose normalized target is in that inventory. |
| `[[#heading]]` tag-like usage | reserved for heading references | Convert known tags to `#[[tag]]`; leave true heading refs as v1 heading refs. |
| `singleton-tag` lint habit | removed | Do not run or preserve singleton-tag fixes; typo detection is `broken-link`. |
| project path config / `scraps_dir` style | `.scraps.toml` directory is wiki root | Move or rewrite config so the file sits at the wiki root. |
| `public/` output assumptions | `_site/` default output | Migrate config, deploy workflows, and docs to `_site/`; do not preserve `public/` as a compatibility branch. |
| `-p` / `--path` / `SCRAPS_PROJECT_PATH` | `-C` / `--directory` / `SCRAPS_DIRECTORY` | Update docs, scripts, CI, and agent settings. |
| template/frontmatter authoring path | skill-based authoring | Remove workflow dependence on generated templates/frontmatter fields; keep ordinary markdown content. |
| `scraps-writer` plugin | `scraps` plugin | Replace `/add-scrap`, `/web-to-scrap`, `/scraps-writer` with `/ingest`; use `/query` for read-side synthesis. |

## Workflow

### 1. Snapshot and scope

1. Check git status. Do not overwrite unrelated user changes.
2. Locate the wiki root and `.scraps.toml`.
3. Count markdown files and identify docs/scripts/CI files that mention
   `scraps`, `scraps-writer`, `-p`, `--path`, `SCRAPS_PROJECT_PATH`,
   `public`, `_site`, `upload-pages-artifact`, `github-pages`, `template`, or
   `frontmatter`.
4. Resolve and record the latest v0 and latest v1-or-newer CLI refs that will
   be used.

### 2. Capture v0 inventory

Run v0 read commands before editing:

```bash
scraps tag list --json
scraps lint
scraps build
```

The tag list is authoritative for v0 implicit tags. Preserve:

- tag names as emitted by v0
- backlink counts
- any scrap/title fields returned by the v0 JSON

If v0 build/lint fails, continue only when `tag list --json` succeeds. Report
the pre-existing failures separately from migration failures.

### 3. Run v1 audit before edits

Run v1:

```bash
scraps lint
scraps build
```

Expect broken links where v0 used implicit tags. Use diagnostics to prioritize
edits, but do not infer tags from diagnostics alone.

### 4. Convert tag syntax

For every markdown file:

1. Find wiki references, excluding fenced code blocks, inline code, autolinks,
   and already-explicit `#[[tag]]`.
2. For each normal `[[target]]`, strip alias and heading parts for comparison:
   - `[[tag|alias]]` compares as `tag`
   - `[[tag#heading]]` compares as `tag`, but keep the heading only if the
     target is a scrap, not a tag
3. If the normalized target exists in the v0 tag inventory and does not resolve
   to a real scrap in v1, convert it to `#[[target]]`.
4. If a target exists both as a v0 tag and a v1 scrap title, do not auto-convert.
   Add it to the report as ambiguous.
5. Do not invent new tags from prose. Do not convert unresolved links that were
   not present in the v0 tag inventory.

Keep aliases only for scrap links. Tags do not use aliases in v1.

### 5. Update config and workflow references

Update `.scraps.toml` and surrounding automation:

- Ensure the directory containing `.scraps.toml` is the wiki root.
- Remove obsolete `scraps_dir`-style configuration if present.
- Migrate build output to `_site/`. Remove `output_dir = "public"` and update
  every dependent workflow/path to `_site`.
- Replace command examples and scripts:
  - `scraps -p <dir> ...` -> `scraps -C <dir> ...`
  - `scraps --path <dir> ...` -> `scraps --directory <dir> ...`
  - `SCRAPS_PROJECT_PATH` -> `SCRAPS_DIRECTORY`
- Update GitHub Pages deployment:
  - Use `boykush/scraps@v1` or a pinned v1 tag/SHA.
  - Ensure the build step runs the v1 CLI, usually `scraps build`.
  - Set `actions/upload-pages-artifact` `path:` to `_site`.
  - Remove `gh-pages` branch deployment assumptions when present.
  - Make the repository Pages setting use **GitHub Actions** as the source.
    If the agent cannot change repository settings directly, report this as a
    required manual post-migration step with the exact setting name.
- Replace legacy plugin instructions:
  - `/add-scrap` -> `/ingest`
  - `/web-to-scrap` -> `/ingest <url>`
  - `/scraps-writer` -> `/ingest`

### 6. Fix v1 diagnostics

Run v1 lint after edits. Handle rules by class:

- `broken-link`: search/read candidate scraps, then either fix the link, convert
  to a v0-known tag, or report as unresolved.
- `broken-heading-ref`: read target headings and update to the closest valid
  heading only when the intended heading is clear.
- `self-link`, `overlinking`: mechanical fixes are allowed.
- `dead-end`, `lonely`: report unless the user asked for graph-shaping edits.

### 7. Verify

Run:

```bash
scraps lint
scraps tag list --json
scraps build
```

Skip `build` only if the wiki intentionally lacks `[ssg]`; say that explicitly.

Compare v0 and v1 tag inventories. Counts may change when nested tags
auto-aggregate, but every v0 tag intentionally preserved should exist in v1
unless reported as renamed/removed.

## Report Format

End with:

- v0 CLI version and v1-or-newer CLI version used
- files changed
- tag conversions count
- config/workflow changes
- GitHub Pages workflow changes and repository setting status
- lint/build result
- ambiguous references or unresolved items needing human judgment

Keep the report concise enough to paste into a PR description.
