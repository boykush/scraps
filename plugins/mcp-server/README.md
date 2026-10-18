# MCP Server Plugin

MCP server for browsing and searching [Scraps](https://github.com/boykush/scraps) wikis.

This plugin points MCP-compatible clients at a locally running `scraps mcp serve --http`, so Scraps tools can be called from any repository on your machine. For most read-shaped agent workflows the simpler path is `scraps <cmd> --json` via the shell — see the [`llm-wiki` plugin](../llm-wiki/README.md) for the bundled CLI + JSON skills.

```text
 scraps -C ~/wiki mcp serve --http    ← one local server, holds the wiki
        │  /mcp
        ├── repo A  (mcp-server plugin)
        ├── repo B  (mcp-server plugin)
        └── repo C  (mcp-server plugin)
```

One server backs every repo: no per-repo wiki path, no MCP subprocess per client. The wiki is read per request through the on-disk IR, so edits are served live and unchanged scraps are not re-parsed.

## Install

### Step 1: Run the server

From anywhere, pointing at the directory that contains `.scraps.toml` (one long-running process):

```bash
scraps -C ~/path/to/your/wiki mcp serve --http
```

It listens on `127.0.0.1:1113` and serves MCP at `http://127.0.0.1:1113/mcp`. Pass an address to `--http` to use a different port or host.

### Step 2: Add the marketplace

```bash
claude plugin marketplace add boykush/scraps
```

### Step 3: Enable the plugin

Add this to your project's `.claude/settings.json`:

```json
{
  "enabledPlugins": {
    "mcp-server@scraps-claude-code-plugins": true
  }
}
```

## Configuration

The plugin connects to `http://127.0.0.1:1113/mcp`. To point at another address — a different port, or a second wiki served by its own process — set `SCRAPS_MCP_URL` before launching your agent:

```bash
scraps -C ~/path/to/another/wiki mcp serve --http 127.0.0.1:1114
export SCRAPS_MCP_URL=http://127.0.0.1:1114/mcp
```

One process serves one wiki. The server binds loopback with no authentication; it is not meant to be exposed to a network.

## MCP tools

All operations run against the current state of the Scraps wiki. Search uses fuzzy matching.

### Naming a scrap

Every result names a scrap with three fields:

```json
{ "key": "tables/orders", "title": "Customer Orders", "ctx": "tables" }
```

`key` is the scrap's path under the wiki root without `.md`, the same string
a wiki-link holds: pass it back to reach that scrap. `title` is what readers
see (the frontmatter `title`, or the file name without `.md`), and `ctx` is
the folder part of the key.

The tools that take a scrap (`get_scrap`, `lookup_scrap_links`,
`lookup_scrap_backlinks`, `lookup_scrap_neighborhood`) take `key`. The older
pair is still accepted in its place: `title`, read as the file name without
`.md` (never the frontmatter title), with an optional `ctx`. Exactly one of
`key` and `title` is required.

### `orient`

Where to start on an unfamiliar wiki: its scale, its folder contexts, the head
of its topic map, and the kinds of scraps it holds in one call. Takes no
parameters.

Returns:

```json
{
  "scrap_count": 128,
  "tag_count": 24,
  "type_count": 2,
  "contexts": ["Book", "Kubernetes"],
  "top_tags": [
    { "title": "rust", "backlinks_count": 31 },
    { "title": "go", "backlinks_count": 12 }
  ],
  "top_types": [
    { "type": "Book", "scraps_count": 40 },
    { "type": "Blog", "scraps_count": 9 }
  ],
  "next": "Search content with search_scraps, expand a top tag with lookup_tag_backlinks {tag}, or expand a top type with lookup_type_scraps {type}; the full lists are list_tags and list_types."
}
```

`contexts` lists every folder context in the wiki, sorted by name. `top_tags`
is capped at 10, ranked by backlink count and tie-broken by title — the whole
list is `list_tags`' job. `top_types` is capped at 10 the same way, in
`list_types`' order (most scraps first, then by name), and the whole list is
`list_types`' job. On a wiki where no scrap declares a type, `type_count` is 0,
`top_types` is empty, and `next` names only the search and tag tools. Bodies
stay out: read a scrap with `get_scrap`. On an empty wiki `next` says so
instead of naming tools.

### `search_scraps`

Search keys, titles and body content with fuzzy matching.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `query` | string | yes | — | Keywords to match |
| `num` | integer | no | 100 | Max results |
| `logic` | `"or"` \| `"and"` | no | `"or"` | Multi-keyword logic |

Returns: `{ results: [{ key, title, ctx, description? }], count }`. `description`
is the scrap's frontmatter description, present only on hits that have one.

### `list_tags`

List all tags with their backlink counts, sorted by popularity.

Returns: `[{ title, backlinks_count }]`.

### `get_scrap`

Retrieve a single scrap by key, with an optional heading and an optional
field projection.

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `key` | string | yes, unless `title` | Scrap key, as results return it |
| `title` | string | no | Still accepted in place of `key`: the file name without `.md` |
| `ctx` | string | no | Still accepted with `title`: context folder/path |
| `heading` | string | no | Restrict body/structure fields to this section |
| `fields` | string[] | no | Defaults to `["key", "title", "ctx", "body"]`; allowed: `key`, `title`, `ctx`, `body`, `headings`, `code_blocks`, `type`, `description`, `frontmatter`, `verified` |

Returns the requested fields. Default response: `{ key, title, ctx, body }`. `type`
and `description` are the scrap's frontmatter type and description, and
`frontmatter` its whole block as parsed data, each `null` when it has none.
`verified` is `{ tier, entries }`: the trust tier (`unverified`,
`machine-confirmed` or `human-reviewed`) and the confirmations newest first.

`body` is the scrap's content without its frontmatter block. Line numbers in
`headings` and `code_blocks` count lines of the source file, block included,
so they are not offsets into `body`; see the
[frontmatter reference](https://boykush.github.io/scraps/scraps/Reference/Frontmatter.html).

### `lookup_scrap_links`

Outbound wiki-links from a scrap.

| Parameter | Type | Required |
|---|---|---|
| `key` | string | yes, unless `title` |
| `title` | string | no (still accepted in place of `key`) |
| `ctx` | string | no (still accepted with `title`) |

Returns outbound reference occurrences:

```json
{
  "results": [
    { "kind": "link", "key": "Target", "title": "Target", "ctx": null, "heading": "Install" },
    { "kind": "embed", "key": "Docs/Guide", "title": "Guide", "ctx": "Docs", "heading": null }
  ],
  "count": 2
}
```

### `lookup_scrap_backlinks`

Inbound wiki-links to a scrap.

| Parameter | Type | Required |
|---|---|---|
| `key` | string | yes, unless `title` |
| `title` | string | no (still accepted in place of `key`) |
| `ctx` | string | no (still accepted with `title`) |

Returns: `{ results: [{ key, title, ctx }], count }`.

### `lookup_scrap_neighborhood`

The neighborhood around a scrap as a graph: everything within a few hops, in
both link directions, with the links between them.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `key` | string | yes, unless `title` | — | Scrap the map opens around |
| `title` | string | no | — | Still accepted in place of `key`: the file name without `.md` |
| `ctx` | string | no | — | Still accepted with `title`: context folder/path |
| `depth` | integer | no | 1 | Hops walked out from the scrap, capped at 5 |
| `limit` | integer | no | 50 | Maximum nodes in the response |

Returns:

```json
{
  "nodes": [
    { "key": "Microservices", "title": "Microservices", "ctx": null, "hop": 0 },
    { "key": "Patterns/Strangler Fig", "title": "Strangler Fig", "ctx": "Patterns", "hop": 1 }
  ],
  "edges": [
    {
      "from": { "key": "Microservices", "title": "Microservices", "ctx": null },
      "to": { "key": "Patterns/Strangler Fig", "title": "Strangler Fig", "ctx": "Patterns" }
    }
  ],
  "count": 2,
  "truncated": false,
  "dropped": 0
}
```

`hop` is the shortest distance from the scrap you asked about, and `edges`
covers every wiki-link between returned nodes, written in link direction. Tags
are not edges — `lookup_tag_backlinks` expands those. Bodies stay out of the
map: read a node with `get_scrap`. When `truncated` is true the node cap cut the
walk short and `dropped` counts what it left out — raise `limit` or lower
`depth`.

### `lookup_tag_backlinks`

Scraps that reference a specific tag.

| Parameter | Type | Required |
|---|---|---|
| `tag` | string | yes |

Returns: `{ results: [{ key, title, ctx }], count }`.

### `list_types`

List the types in use with the number of scraps of each, most used first. A
type is the `type` key of a scrap's frontmatter. Takes no parameters.

Returns: `{ results: [{ type, scraps_count }], count }`.

### `lookup_type_scraps`

Scraps of a specific type. The type is matched exactly and case-sensitively.

| Parameter | Type | Required |
|---|---|---|
| `type` | string | yes |

Returns: `{ results: [{ key, title, ctx }], count }`.

### `list_todos`

Markdown task list items aggregated across every scrap.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `status` | `"open"` \| `"done"` \| `"deferred"` \| `"all"` | no | `"open"` | Task status filter |

Returns:

```json
{
  "results": [
    {
      "scrap": { "key": "Programming/Rust/borrowing", "title": "borrowing", "ctx": "Programming/Rust" },
      "status": "open",
      "text": "implement Drop for X",
      "line": 3
    }
  ],
  "count": 1
}
```

`- [ ]` is `open`, `- [x]` is `done` and `- [-]` is `deferred`; any other symbol
is skipped, and items inside fenced code blocks are not tasks. Results are
ordered by context, then file name, then line, and carry the same shape as
`scraps todo --json`. Read the scrap a task sits in with `get_scrap`.

### `list_frontmatter`

YAML frontmatter aggregated across every scrap that carries a block. Takes no
parameters.

Returns:

```json
{
  "results": [
    {
      "scrap": { "key": "Programming/Rust/borrowing", "title": "borrowing", "ctx": "Programming/Rust" },
      "frontmatter": { "status": "review", "owner": "bob", "taxonomy": ["rust", "memory"] }
    }
  ],
  "count": 1
}
```

Keys are passed through in the order they were written, and Scraps requires
none of them. It reads five: `title` is the scrap's title (the `title` of
every result), the strings of a `tags` list are tags alongside
`#[[tag]]` (`list_tags`, `lookup_tag_backlinks`), `type` is the scrap's
type (`list_types`, `lookup_type_scraps`), `description` is its one-line
summary (`get_scrap`, `search_scraps`), and `verified` its confirmations
(`get_scrap`). Every other key is opaque — see the
[frontmatter reference](https://boykush.github.io/scraps/scraps/Reference/Frontmatter.html).
Scraps without a block are omitted, and a malformed block is skipped rather
than failing the call.

## Manual setup (without the plugin)

Register the shared server with any MCP-compatible client:

```bash
claude mcp add --transport http scraps http://127.0.0.1:1113/mcp
```

### stdio (no server to run)

For a single repository, the client can spawn `scraps mcp serve` as a subprocess instead:

```bash
claude mcp add scraps -- scraps -C ~/path/to/your/wiki mcp serve
```

Replace `~/path/to/your/wiki` with the directory containing `.scraps.toml`. stdio needs that path per repository, so — unlike the shared HTTP URL — it cannot ship as a turnkey plugin config. Plugin versions before 2.0.0 bundled this stdio setup; if you relied on it, either start the server (see [Install](#install)) or add the command above.

## Further reading

- Scraps documentation: <https://boykush.github.io/scraps/>
- AI integration overview: <https://boykush.github.io/scraps/scraps/How-to/Integrate%20with%20AI%20Assistants.html>
- CLI + JSON skills bundle: [`llm-wiki` plugin](../llm-wiki/README.md)
