---
type: Reference
---

#[[CLI]]

Scraps is a CLI-first compiler. Every command supports `--help`, which is
the authoritative reference for flags and arguments. This page is the
**map**. Every command that reads scraps goes through the on-disk
[[Reference/IR]], so repeated queries never re-parse unchanged sources.

| Command | Role | `--json` |
|---|---|---|
| `scraps init` | Write `.scraps.toml` to the current directory | – |
| `scraps build` | Compile to the `_site/` static site | – |
| `scraps check` | Run the build without writing `_site/`; fails where `scraps build` would | – |
| `scraps serve` | Build then serve at `http://127.0.0.1:1112` | – |
| `scraps lint` | Wiki-link health check ([[Reference/Lint Rules]]) | – |
| `scraps get <key>` | Single-scrap introspection | ✓ |
| `scraps search <query>` | Fuzzy search over keys, titles + body | ✓ |
| `scraps links <key>` | Outbound wiki-links from a scrap | ✓ |
| `scraps backlinks <key>` | Inbound wiki-links to a scrap | ✓ |
| `scraps tag list` | List all tags with backlink counts | ✓ |
| `scraps tag backlinks <tag>` | Scraps referencing a tag | ✓ |
| `scraps type list` | List all types with scrap counts | ✓ |
| `scraps type scraps <type>` | Scraps of a type | ✓ |
| `scraps todo` | Aggregate GFM task list items wiki-wide | ✓ |
| `scraps frontmatter` | Aggregate YAML frontmatter wiki-wide | ✓ |
| `scraps mcp serve` | Start an MCP server over stdio, or `--http` | – |

`-C` / `--directory` (or `SCRAPS_DIRECTORY` env) runs as if started in the
given directory.

`scraps build` and `scraps serve` read each scrap's last commit date from
git; `--no-git` skips it (see [[Reference/Static Site/Sort and Pagination]]).

## Naming a scrap

`get`, `links` and `backlinks` take a scrap's key, the same string a
wiki-link holds ([[Reference/Wiki-link Notation#Key, title and context]]):

```bash
scraps get "Tutorial/Getting Started"
scraps links tables/orders --json
```

Every `--json` output that names a scrap carries three fields: `key` to pass
back to a command, `title` to show a reader, and `ctx`, the folder part of
the key.

```json
{ "key": "tables/orders", "title": "Customer Orders", "ctx": "tables" }
```

`--ctx` is still supported: with it the argument is the file name without
`.md` inside that context, so `scraps get orders --ctx tables` reads the
same scrap. Neither form looks a scrap up by its frontmatter title; find the
key with `scraps search`.

## JSON Reads

`scraps get` reads one scrap, optionally scoped by heading:

```bash
scraps get "Book/Title" --heading "Install" --json
```

Its `--json` flag accepts an optional comma-separated field projection. With
no projection it returns `key`, `title`, `ctx`, and `body`.

```bash
scraps get "Book/Title" --json key,title,body
scraps get "Book/Title" --json code_blocks
scraps get "Book/Title" --json images
scraps get "Book/Title" --heading "Install" --json body,headings
```

Allowed fields are `key`, `title`, `ctx`, `body`, `headings`, `code_blocks`,
`images`, `type`, `description`, `frontmatter`, `verified`, `sources`, and `resource`. `type` and `description` are the
scrap's frontmatter type and description, and `frontmatter` its whole block,
each `null` when it has none. `verified` is the scrap's trust tier and
confirmations, `{ tier, entries }`, `sources` the materials it
derives from, and `resource` what it is about, `null` when it has none. What `body` leaves out and what the line
numbers in `headings` and `code_blocks` count is in
[[Reference/Frontmatter]].
Reference navigation stays separate: use `scraps links`, `scraps backlinks`,
and `scraps tag ...`.

`scraps links --json` returns outbound reference occurrences. Each result has
`kind` (`link`, `embed`, or `source` for a frontmatter `sources` path), `key`,
`title`, `ctx`, and `heading`, so agents can jump directly to
`scraps get <key> --heading <heading>`.

`scraps backlinks --json` remains scrap-level inbound discovery and returns
the scraps that link to the requested scrap, each with `kinds`: `link` when
its body links here, `source` when it names this scrap in its `sources`.

`scraps type list --json` returns each type in use with its `scraps_count`,
most used first. `scraps type scraps <type> --json` returns the scraps of one
type; the type is matched exactly and case-sensitively.

`scraps frontmatter --json` returns every scrap that carries a YAML
frontmatter block, with all of its keys as written. Which keys Scraps reads,
and how, is in [[Reference/Frontmatter]].

For agent integration, see [[How-to/Integrate with AI Assistants]].
