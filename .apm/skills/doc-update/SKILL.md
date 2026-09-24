---
name: doc-update
description: Check Scraps' documentation against what a release shipped, propose the updates it is missing, and apply the ones the user picks. Use this after a release is published (e.g. "/doc-update 3.1.0", "v3.1.0 の docs を見直して"), or whenever the user asks whether the docs site, README or plugin READMEs still match released behaviour.
---

# Doc update

Feature PRs normally update their docs in the same change. This skill looks for what slipped through anyway:

- behaviour the release changed that the docs still describe the old way
- pages where a new feature should appear but doesn't
- screenshots that no longer match

Takes a version such as `3.1.0`. Without one, use the latest release (`gh release view --json tagName`).

## 1. Read what shipped

- `gh release view v<version> --json body` lists the PRs by section, with breaking changes first.
- Keep what users would notice: breaking changes, features, fixes, and performance changes. Skip maintenance.
- For each PR you keep, `gh pr view <N> --json files --jq '.files[].path'` shows which docs it already updated. Don't propose those changes again.

## 2. Find where each change belongs

| Surface | Where | Holds |
| --- | --- | --- |
| Docs site | `docs/` | A Scraps wiki laid out by Diátaxis in `Tutorial/`, `How-to/`, `Reference/` and `Explanation/`. `docs/README.md` is the index page |
| README | `README.md` | Quick start, how Scraps works, AI integration, screenshots |
| MCP tool reference | `plugins/mcp-server/README.md` | Parameters and response shape of every MCP tool |
| Plugin overviews | `plugins/*/README.md` | What each plugin bundles |
| Screenshots | `assets/` | Images the README embeds |

Pages move between releases, so list the tree (`git ls-files docs README.md plugins assets`) rather than assuming paths. To find everything that mentions a changed command, flag, config key or term, run `git grep -n "<term>" -- docs README.md plugins`. An exact match catches stale mentions that fuzzy search misses.

Usual homes for a change:

- A command, flag or `--json` field: `docs/Reference/CLI Overview.md`. If agents use it, also `docs/How-to/Integrate with AI Assistants.md`.
- A config key: `docs/Reference/Configuration.md`
- A lint rule: `docs/Reference/Lint Rules.md`
- An MCP tool: `plugins/mcp-server/README.md`
- A change in what CI needs, such as checkout depth or action inputs: `docs/How-to/Deploy to GitHub Pages.md`
- A new major version: every `boykush/scraps@v<major>` reference

## 3. Propose

Group the proposals by file. For each one, give what to add or change, which section it goes in, and the PR that calls for it. List first anything that still describes the old behaviour of a breaking change. Then ask the user which proposals to apply.

## 4. Apply what the user picks

- Match the page's voice and its wiki-links (`[[Reference/CLI Overview]]`, `#[[Tag]]`). Link a new page from the index or from its parent page. For example, pages under `Reference/Static Site/` are linked from `Reference/Static Site.md`.
- Check links with the binary built from this checkout: `cargo build --release && ./target/release/scraps -C docs lint -r broken-link -r broken-heading-ref`.
- Open the edits as their own docs PR (`docs: ...`), separate from the release.
