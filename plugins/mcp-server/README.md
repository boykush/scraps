# MCP Server Plugin

MCP server for browsing and searching [Scraps](https://github.com/boykush/scraps) wikis.

This plugin packages `scraps mcp serve` as a Claude Code plugin so MCP-compatible clients can call Scraps tools directly. For most read-shaped agent workflows the simpler path is `scraps <cmd> --json` via the shell — see the [`scraps` plugin](../scraps/README.md) for the bundled CLI + JSON skills.

## Install

### Step 1: Add the marketplace

```bash
claude plugin marketplace add boykush/scraps
```

### Step 2: Enable the plugin

Add this to your project's `.claude/settings.json`:

```json
{
  "enabledPlugins": {
    "mcp-server@scraps-claude-code-plugins": true
  }
}
```

The plugin uses the current directory as the Scraps wiki root. To target a different wiki, set `SCRAPS_DIRECTORY`:

```json
{
  "env": {
    "SCRAPS_DIRECTORY": "/path/to/your/wiki"
  },
  "enabledPlugins": {
    "mcp-server@scraps-claude-code-plugins": true
  }
}
```

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

Retrieve a single scrap by title (and optional context).

| Parameter | Type | Required |
|---|---|---|
| `title` | string | yes |
| `ctx` | string | no |

Returns: `{ title, ctx, md_text, headings, code_blocks }`.

### `lookup_scrap_links`

Outbound wiki-links from a scrap.

| Parameter | Type | Required |
|---|---|---|
| `title` | string | yes |
| `ctx` | string | no |

Returns: `{ results: [{ title, ctx }], count }`.

### `lookup_scrap_backlinks`

Inbound wiki-links to a scrap.

| Parameter | Type | Required |
|---|---|---|
| `title` | string | yes |
| `ctx` | string | no |

Returns: `{ results: [{ title, ctx }], count }`.

### `lookup_tag_backlinks`

Scraps that reference a specific tag.

| Parameter | Type | Required |
|---|---|---|
| `tag` | string | yes |

Returns: `{ results: [{ title, ctx }], count }`.

## Manual setup (without the plugin)

For other MCP-compatible clients, run the server directly:

```bash
claude mcp add scraps -- scraps -C ~/path/to/your/wiki mcp serve
```

Replace `~/path/to/your/wiki` with the directory containing `.scraps.toml`.

## Further reading

- Scraps documentation: <https://boykush.github.io/scraps/>
- AI integration overview: <https://boykush.github.io/scraps/scraps/how-to/integrate-with-ai-assistants.html>
- CLI + JSON skills bundle: [`scraps` plugin](../scraps/README.md)
