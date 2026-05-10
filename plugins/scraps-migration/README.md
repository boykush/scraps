# Scraps Migration Plugin

LLM-facing migration workflow for moving Scraps wikis from v0 to v1.

This plugin is intentionally workflow-only: it does not add a migration command
to the Scraps binary. Instead, it gives Claude Code and Codex a shared skill for
running the old and new CLIs side by side, applying controlled markdown/config
edits, and verifying the upgraded wiki with v1 lint/build.

## Install

### Claude Code

```bash
claude plugin marketplace add boykush/scraps
```

Then enable:

```json
{
  "enabledPlugins": {
    "scraps-migration@scraps-claude-code-plugins": true
  }
}
```

### Codex

Install this plugin from the same repository/plugin directory. The shared skill
body lives at `skills/migrate-v0-to-v1/SKILL.md`; the Codex manifest is
`plugins/scraps-migration/.codex-plugin/plugin.json`.

## Skill

### `/migrate-v0-to-v1 [wiki-root]`

Audit and migrate a wiki from Scraps v0 to v1.

The skill:

- captures v0 inventory with the v0 CLI, especially `scraps tag list --json`
- runs v1 lint/build to identify breaking changes
- converts known v0 tag links to explicit v1 `#[[tag]]`
- updates `.scraps.toml` discovery/config shape
- removes `scraps template generate` / `scraps template list` workflow usage
- replaces `-p` / `--path` workflow references with `-C` / `--directory`
- fully migrates GitHub Pages deploys to v1 (`_site/`, `boykush/scraps@v1`,
  and Pages source set to GitHub Actions)
- validates the final wiki with v1 `scraps lint` and, when `[ssg]` exists,
  `scraps build`

## Version discovery

The skill does not require users to supply version inputs. It discovers the
latest v0 and latest v1-or-newer refs from repository tags when possible, then
runs version pinned CLIs with `mise`.

At the time this plugin was written, the latest known tags were:

- v0: `v0.33.0`
- v1-or-newer: `v1.0.0-rc.2`

Example execution:

```bash
mise exec github:boykush/scraps@v0.33.0 -- scraps -C <wiki-root> tag list --json
mise exec github:boykush/scraps@v1.0.0-rc.2 -- scraps -C <wiki-root> lint
```

If tags are unavailable and v0 cannot be inferred from the wiki/repo, the skill
asks for the old version before making migration edits because v0
`scraps tag list --json` is the safe source of truth for implicit tags.

## Further Reading

- v1 vision: `design/v1/vision.md`
- v1 AI skills: `plugins/scraps/README.md`
- legacy v0 skills: `plugins/scraps-writer/README.md`
