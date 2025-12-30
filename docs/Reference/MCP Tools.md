#[[Integration]]

This reference documents the MCP (Model Context Protocol) tools provided by
Scraps for AI assistant integration.

## search_scraps

Search through your Scraps content with natural language queries using fuzzy
matching.

**Parameters:**
- `query` (string, required): Search query to match against scrap titles and
  contexts
- `num` (integer, optional): Maximum number of results to return (default: 100)

**Returns:**
- `results`: Array of matching scraps with their title, context, and full
  Markdown content
- `count`: Total number of matches found

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
Array of linked scraps with their full content.

## lookup_scrap_backlinks

Find inbound wiki links (backlinks) to a specific scrap. Returns all scraps
that link to the specified scrap.

**Parameters:**
- `title` (string, required): Title of the scrap to get backlinks for
- `ctx` (string, optional): Context if the scrap has one

**Returns:**
Array of scraps that link to the specified scrap, with their full content.

## lookup_tag_backlinks

Find all scraps that reference a specific tag.

**Parameters:**
- `tag` (string, required): Tag name to get backlinks for

**Returns:**
Array of scraps that reference the specified tag, with their full content.

## Notes

- All search and lookup operations are performed against the current state of
  your Scraps repository
- Fuzzy matching is used for search queries to improve discoverability
- Results include the full Markdown content of matching scraps
- The MCP server must be running for these tools to be available to your AI
  assistant

For setup instructions, see [[How-to/Integrate with AI Assistants]].
