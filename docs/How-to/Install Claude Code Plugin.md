#[[Integration]]

This guide shows you how to enable the Scraps MCP (Model Context Protocol)
server plugin in Claude Code for seamless AI assistant interaction with your
Scraps knowledge base.

## Configuration

Add the following to your `.claude/settings.json`:

```json
{
  "enabledPlugins": {
    "scraps-claude-code-plugins@mcp-server": true
  }
}
```

The plugin will automatically use the current directory as your Scraps project path.

## Custom Project Path (Optional)

To specify a different Scraps project path, set the `SCRAPS_PROJECT_PATH`
environment variable:

```bash
export SCRAPS_PROJECT_PATH="/path/to/your/scraps/project"
```
