# Scraps Writer Plugin (DEPRECATED)

> **Deprecated.** This plugin is the v0 skill bundle, kept temporarily for users mid-migration. Use the [`scraps` plugin](../scraps/README.md) instead.
>
> Scheduled for removal in a future release.

## Why deprecated

The v1 redesign moved the official AI skills bundle to the new [`scraps` plugin](../scraps/README.md), which:

- runs through `scraps <cmd> --json` instead of MCP, so any agent that can run a shell can use it
- maps directly to Karpathy's *Ingest / Query / Lint* primitives
- ships a `lint-rule-handler` agent for purpose-driven wiki health checks

The legacy `scraps-writer` skills are still functional via the bundled MCP server, but they will not receive feature updates.

## Migration guide

| Legacy (`scraps-writer`) | v1 (`scraps`) |
| --- | --- |
| `/add-scrap "<title>"` | `/ingest "<topic or instruction>"` |
| `/web-to-scrap <url>` | `/ingest <url>` |
| `/scraps-writer "<title>"` | `/ingest "<topic>"` |
| (lint built into add-scrap) | invoke the `lint-rule-handler` agent with a stated purpose |
| (no read-side workflow) | `/query "<question>"` |

To migrate:

1. Install the new `scraps` plugin from the same marketplace.
2. Replace any saved aliases or workflow scripts that reference `/add-scrap` / `/web-to-scrap` with `/ingest`.
3. Uninstall `scraps-writer` once you have confirmed the new flows.

## Legacy skills (still functional during the transition)

### `/add-scrap [title] [max-lines]`

Create a new scrap on any topic. Researches existing tags and related scraps, drafts the new scrap, and suggests backlinks.

### `/web-to-scrap [url] [max-lines]`

Convert a web article into a scrap. Fetches the article, generates a concise summary, adds a source autolink, and connects the content via tags and `[[Wiki-links]]`.

### `/scraps-writer [title] [max-lines]`

Shared workflow used by `add-scrap` and `web-to-scrap` for tag research, Wiki-link resolution, and content verification.

## Further reading

- New `scraps` plugin: [../scraps/README.md](../scraps/README.md)
- Scraps documentation: <https://boykush.github.io/scraps/>
