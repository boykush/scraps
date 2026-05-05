#[[Integration]]

Scraps includes comprehensive Model Context Protocol (MCP) server functionality, enabling AI assistants to directly interact with your Scraps knowledge base.

## What is MCP?

The Model Context Protocol (MCP) is an open standard that enables AI assistants to securely access external data sources and tools. Scraps implements an MCP server that exposes your documentation as a searchable, linkable knowledge base.

## Quick Start

### Claude Code (Recommended)

For Claude Code users, we provide an official plugin for seamless integration. See [[How-to/Install Claude Code Plugin]] for installation instructions.

### Manual MCP Server Setup

For other MCP-compatible clients or advanced configurations, you can add Scraps as an MCP server directly:

```bash
claude mcp add scraps -- scraps mcp serve --path ~/path/to/your/scraps/project/
```

Replace `~/path/to/your/scraps/project/` with the actual path to your Scraps project directory.

For command details, see [[Reference/MCP Serve]].

## Available Tools

For detailed MCP tool documentation, see [[Reference/MCP Tools]].
