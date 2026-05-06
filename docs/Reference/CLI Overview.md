#[[CLI]]

Scraps is a CLI-first compiler. Every command supports `--help`, which is the
authoritative reference for flags and arguments. This page is the **map** —
what each command does, when to use it, and which ones speak `--json` for
agents.

For the design rationale behind CLI + JSON as the primary agent integration,
see [[Explanation/What is Scraps?#cli-first-for-agents]].

## Commands

| Command | Role | `--json` |
|---|---|---|
| `scraps init` | Write `.scraps.toml` to the current directory | – |
| `scraps build` | Compile to the `_site/` static site (see [[Reference/Static Site]]) | – |
| `scraps serve` | Build then serve at `http://127.0.0.1:1112` | – |
| `scraps lint` | Wiki-link health check (default: graph-mechanical rules) | – |
| `scraps get <title>` | Single-scrap introspection | ✓ |
| `scraps search <query>` | Fuzzy search over titles + body | ✓ |
| `scraps links <title>` | Outbound wiki-links from a scrap | ✓ |
| `scraps backlinks <title>` | Inbound wiki-links to a scrap | ✓ |
| `scraps tag list` | List all tags with backlink counts | ✓ |
| `scraps tag backlinks <tag>` | List scraps referencing a tag | ✓ |
| `scraps todo` | Aggregate GFM task list items across the wiki | ✓ |
| `scraps mcp serve` | Start an MCP server (see the [mcp-server plugin](https://github.com/boykush/scraps/tree/main/plugins/mcp-server)) | – |

Run `scraps <command> --help` for the full flag list and shape.

## --json

Read commands emit structured JSON when invoked with `--json`. This is the
primary path for agent integration: any assistant that can run a shell command
can query Scraps without an MCP client implementation.

```bash
❯ scraps search "release checklist" --json
❯ scraps get "Configuration" --json
❯ scraps backlinks "Configuration" --json
❯ scraps tag list --json
❯ scraps todo --status all --json
```

JSON shapes are stable per command; see each command's `--help` for the schema.
For the agent-side workflow, see [[How-to/Integrate with AI Assistants]].

## Common options

`-C` / `--directory` runs as if started in the given directory. The
`SCRAPS_DIRECTORY` environment variable does the same. Both let you target a
specific wiki when multiple `.scraps.toml` exist in one repository.

```bash
❯ scraps -C path/to/wiki build
❯ SCRAPS_DIRECTORY=path/to/wiki scraps lint
```

The `-p` / `--path` flag from earlier versions is still accepted as a
deprecated alias and will be removed in a future release.

## lint

`scraps lint` is the single wiki-health entry point. Default rules are
graph-mechanical (no external dependencies); rules with dependencies are
opt-in via `--rule` or `[lint.<rule>]` in
[[Reference/Configuration#lint-rules]].

| Rule | Type | Default |
|---|---|---|
| `dead-end` | graph: scrap with no outbound links | on |
| `lonely` | graph: scrap with no backlinks | on |
| `self-link` | graph: scrap links to itself | on |
| `overlinking` | graph: same `[[link]]` repeated in one scrap | on |
| `broken-link` | graph: `[[link]]` does not resolve | on |
| `broken-heading-ref` | graph: `[[Page#Heading]]` heading missing | on |
| `stale-by-git` | git-dependent: last commit older than threshold | opt-in |

Output follows the `cargo clippy`-style diagnostic format. For LLM-driven
purpose-based rule selection, see the `lint-rule-handler` agent in the
[scraps plugin](https://github.com/boykush/scraps/tree/main/plugins/scraps).

## build / serve

Both write to the directory configured in
[[Reference/Configuration#root-level]] (`output_dir`, default `_site/`).
Output behavior, README handling, and search index are documented in
[[Reference/Static Site]].

`--git` opt-in flag adds git-derived `committed_ts` metadata to the build
output; commands that intrinsically require git (none in v1) would not need
the flag.

## todo

GFM task list items aggregated across the wiki. Status filters: `open`
(`- [ ]`), `done` (`- [x]`), `deferred` (`- [-]`), `all`. Default is `open`.

```bash
❯ scraps todo
❯ scraps todo --status all --json
```
