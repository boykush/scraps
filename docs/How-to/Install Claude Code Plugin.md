#[[Integration]]

This guide shows you how to enable the Scraps MCP (Model Context Protocol)
server plugin in Claude Code.

## Installation

### Step 1: Add the Plugin Marketplace

First, add the Scraps plugin marketplace:

```bash
claude plugin marketplace add boykush/scraps
```

This registers the Scraps plugin catalog with Claude Code.

### Step 2: Enable the Plugin

Add the following to your project's `.claude/settings.json`:

```json
{
  "enabledPlugins": {
    "mcp-server@scraps-claude-code-plugins": true
  }
}
```

The plugin will automatically use the current directory as your Scraps project
path.

## Configuration

### Custom Project Path (Optional)

To specify a different Scraps project path, set the `SCRAPS_PROJECT_PATH`
environment variable:

```json
{
  "env": {
    "SCRAPS_PROJECT_PATH": "/path/to/your/scraps/project"
  },
  "enabledPlugins": {
    "mcp-server@scraps-claude-code-plugins": true
  }
}
```
