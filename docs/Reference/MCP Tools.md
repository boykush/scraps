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
