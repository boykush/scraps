#[[Integration]]

Scraps includes comprehensive Model Context Protocol server functionality, enabling AI assistants like Claude Code to directly interact with your Scraps knowledge base.

## Quick Start with Claude Code

The fastest way to get started is with Claude Code. Add Scraps as a Model Context Protocol server with a single command:

```bash
claude mcp add scraps -- scraps mcp serve --path ~/path/to/your/scraps/project/
```

Replace `~/path/to/your/scraps/project/` with the actual path to your Scraps project directory.

## Available Tools

- **search_scraps**: Search through your Scraps content with natural language queries
- **list_tags**: List all available tags in your Scraps repository
- **lookup_scrap_links**: Find outbound wiki links from a specific scrap
- **lookup_scrap_backlinks**: Find scraps that link to a specific scrap
- **lookup_tag_backlinks**: Find all scraps that reference a specific tag
