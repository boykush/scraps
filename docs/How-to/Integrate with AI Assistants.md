#[[Integration]]

Scraps is CLI-first for AI integration. Prefer `--json` commands when your
assistant can run shell commands, and use the MCP server when your client expects
Model Context Protocol tools.

## CLI JSON

Any assistant with shell access can query Scraps without a long-running server:

```bash
❯ scraps search "rust cli" --logic and --json
❯ scraps get "Getting Started" --json
❯ scraps links "Getting Started" --json
❯ scraps backlinks "Configuration" --json
❯ scraps tag list --json
❯ scraps todo --status all --json
```

## What is MCP?

The Model Context Protocol (MCP) is an open standard that enables AI assistants to securely access external data sources and tools. Scraps implements an MCP server that exposes your documentation as a searchable, linkable knowledge base.

## Quick Start

### Claude Code (Recommended)

For Claude Code users, we provide an official plugin for seamless integration. See [[How-to/Install Claude Code Plugin]] for installation instructions.

### Manual MCP Server Setup

For other MCP-compatible clients or advanced configurations, you can add Scraps as an MCP server directly:

```bash
claude mcp add scraps -- scraps -C ~/path/to/your/wiki mcp serve
```

Replace `~/path/to/your/wiki` with the directory containing `.scraps.toml`.

For command details, see [[Reference/MCP Serve]].

## Available Tools

For detailed MCP tool documentation, see [[Reference/MCP Tools]]. For CLI JSON
commands, see [[Reference/Get]], [[Reference/Search]], [[Reference/Links]], [[Reference/Backlinks]], [[Reference/Tag]], and [[Reference/Todo]].
