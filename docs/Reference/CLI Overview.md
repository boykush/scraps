#[[CLI]]

Scraps is a CLI-first compiler. Every command supports `--help`, which is
the authoritative reference for flags and arguments. This page is the
**map**.

| Command | Role | `--json` |
|---|---|---|
| `scraps init` | Write `.scraps.toml` to the current directory | – |
| `scraps build` | Compile to the `_site/` static site | – |
| `scraps serve` | Build then serve at `http://127.0.0.1:1112` | – |
| `scraps lint` | Wiki-link health check ([[Reference/Lint Rules]]) | – |
| `scraps get <title>` | Single-scrap introspection | ✓ |
| `scraps search <query>` | Fuzzy search over titles + body | ✓ |
| `scraps links <title>` | Outbound wiki-links from a scrap | ✓ |
| `scraps backlinks <title>` | Inbound wiki-links to a scrap | ✓ |
| `scraps tag list` | List all tags with backlink counts | ✓ |
| `scraps tag backlinks <tag>` | Scraps referencing a tag | ✓ |
| `scraps todo` | Aggregate GFM task list items wiki-wide | ✓ |
| `scraps mcp serve` | Start an MCP server | – |

`-C` / `--directory` (or `SCRAPS_DIRECTORY` env) runs as if started in the
given directory.

For agent integration, see [[How-to/Integrate with AI Assistants]].
