#[[Integration]]

This reference documents the MCP (Model Context Protocol) tools provided by
Scraps for AI assistant integration.

## search_scraps

Search through your Scraps content with natural language queries using fuzzy
matching.

**Parameters:**
- `query` (string, required): Search query to match against scrap titles and
  body content
- `num` (integer, optional): Maximum number of results to return (default: 100)
- `logic` (string, optional): Search logic for combining multiple keywords:
  - `"or"` (default): Any keyword can match
  - `"and"`: All keywords must match

**Returns:**
- `results`: Array of matching scraps with the following fields:
  - `title`: Scrap title
  - `ctx`: Context folder path (null if in root)
- `count`: Total number of matches found

**Examples:**
- `{"query": "rust cli", "logic": "and"}` - Returns scraps containing both
  "rust" AND "cli"
- `{"query": "rust cli", "logic": "or"}` - Returns scraps containing "rust" OR
  "cli"

## get_scrap

Get a single scrap by title and optional context. Returns the full Markdown
content together with extracted structural data (headings, fenced code blocks)
so AI assistants can introspect a scrap in a single call without re-parsing.

**Parameters:**
- `title` (string, required): Title of the scrap to get
- `ctx` (string, optional): Context if the scrap has one

**Returns:**
- `title`: Scrap title
- `ctx`: Context folder path (null if in root)
- `md_text`: Full Markdown body
- `headings`: Array of headings in occurrence order, each with:
  - `level`: ATX/Setext heading level (1–6)
  - `text`: Plain-text label (inline markup and wiki-link aliases collapsed)
  - `line`: 1-based source line of the heading
  - `parent` (optional): Text of the closest enclosing heading at a lower
    level. Omitted for top-level headings
- `code_blocks`: Array of fenced code blocks in occurrence order, each with:
  - `lang`: Language tag from the info string, or `null` when none
  - `content`: Raw block body
  - `line`: 1-based source line of the opening fence

Indented (4-space) code blocks are intentionally excluded — only fenced
blocks carry an explicit language tag.

Future structural fields (tables, callouts, images, …) will follow the same
pattern: a top-level array on the response keyed by the construct name,
with one object per occurrence carrying its semantic attributes plus a
`line` anchor back into `md_text`.

## list_tags

List all available tags in your Scraps repository with their backlink counts,
sorted by popularity.

**Parameters:** None

**Returns:**
Array of tags with the following fields:
- `title`: Tag name
- `backlinks_count`: Number of scraps referencing this tag

## lookup_scrap_links

Find outbound wiki links from a specific scrap. Returns all scraps that the
specified scrap links to.

**Parameters:**
- `title` (string, required): Title of the scrap to get links for
- `ctx` (string, optional): Context if the scrap has one

**Returns:**
Array of linked scraps with `title` and `ctx`.

## lookup_scrap_backlinks

Find inbound wiki links (backlinks) to a specific scrap. Returns all scraps
that link to the specified scrap.

**Parameters:**
- `title` (string, required): Title of the scrap to get backlinks for
- `ctx` (string, optional): Context if the scrap has one

**Returns:**
Array of scraps that link to the specified scrap, with `title` and `ctx`.

## lookup_tag_backlinks

Find all scraps that reference a specific tag.

**Parameters:**
- `tag` (string, required): Tag name to get backlinks for

**Returns:**
Array of scraps that reference the specified tag, with `title` and `ctx`.

## Notes

- All search and lookup operations are performed against the current state of
  your Scraps repository
- Fuzzy matching is used for search queries to improve discoverability
- Use `get_scrap` to retrieve the full Markdown content of individual scraps
- The MCP server must be running for these tools to be available to your AI
  assistant

For setup instructions, see [[How-to/Integrate with AI Assistants]].
