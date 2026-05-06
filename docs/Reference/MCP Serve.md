#[[CLI]]

```bash
❯ scraps mcp serve
```

This command starts an MCP (Model Context Protocol) server that enables AI assistants like Claude Code to directly interact with your Scraps knowledge base.

## Examples

```bash
# Basic MCP server
❯ scraps mcp serve

# Serve from specific directory
❯ scraps -C /path/to/wiki mcp serve

```

The MCP server provides tools for AI assistants to search content, retrieve
scraps, inspect wiki-links, and list tags. CLI JSON commands are the primary
agent integration surface in v1; MCP remains available for MCP-compatible
clients.

For more details, see [[How-to/Integrate with AI Assistants]].
