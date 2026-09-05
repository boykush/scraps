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

One server backs every repo: no per-repo wiki path, no MCP subprocess per client. The wiki is read per request, so edits are served live.

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

### `search_scraps`

Search titles + body content with fuzzy matching.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `query` | string | yes | — | Keywords to match |
| `num` | integer | no | 100 | Max results |
| `logic` | `"or"` \| `"and"` | no | `"or"` | Multi-keyword logic |

Returns: `{ results: [{ title, ctx }], count }`.

### `list_tags`

List all tags with their backlink counts, sorted by popularity.

Returns: `[{ title, backlinks_count }]`.

### `get_scrap`

Retrieve a single scrap by title, optional context, optional heading, and
optional field projection.

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `title` | string | yes | Scrap title |
| `ctx` | string | no | Context folder/path |
| `heading` | string | no | Restrict body/structure fields to this section |
| `fields` | string[] | no | Defaults to `["title", "ctx", "body"]`; allowed: `title`, `ctx`, `body`, `headings`, `code_blocks` |

Returns the requested fields. Default response: `{ title, ctx, body }`.

### `lookup_scrap_links`

Outbound wiki-links from a scrap.

| Parameter | Type | Required |
|---|---|---|
| `title` | string | yes |
| `ctx` | string | no |

Returns outbound reference occurrences:

```json
{
  "results": [
    { "kind": "link", "title": "Target", "ctx": null, "heading": "Install" },
    { "kind": "embed", "title": "Guide", "ctx": "Docs", "heading": null }
  ],
  "count": 2
}
```

### `lookup_scrap_backlinks`

Inbound wiki-links to a scrap.

| Parameter | Type | Required |
|---|---|---|
| `title` | string | yes |
| `ctx` | string | no |

Returns: `{ results: [{ title, ctx }], count }`.

### `lookup_scrap_neighborhood`

The neighborhood around a scrap as a graph: everything within a few hops, in
both link directions, with the links between them.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `title` | string | yes | — | Scrap the map opens around |
| `ctx` | string | no | — | Context folder/path |
| `depth` | integer | no | 1 | Hops walked out from the scrap, capped at 5 |
| `limit` | integer | no | 50 | Maximum nodes in the response |

Returns:

```json
{
  "nodes": [
    { "title": "Microservices", "ctx": null, "hop": 0 },
    { "title": "Strangler Fig", "ctx": null, "hop": 1 }
  ],
  "edges": [
    {
      "from": { "title": "Microservices", "ctx": null },
      "to": { "title": "Strangler Fig", "ctx": null }
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

Returns: `{ results: [{ title, ctx }], count }`.

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
- AI integration overview: <https://boykush.github.io/scraps/scraps/how-to/integrate-with-ai-assistants.html>
- CLI + JSON skills bundle: [`llm-wiki` plugin](../llm-wiki/README.md)
