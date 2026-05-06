# Scraps Plugin

Official AI skills bundle for [Scraps](https://github.com/boykush/scraps), the Wiki-link document compiler for the LLM era.

## Overview

This plugin provides skills and an agent that wrap the `scraps` CLI for AI-driven workflows. All integration runs through `scraps <cmd> --json`; no MCP dependency. For users who prefer an MCP server, see the separate [`mcp-server`](../mcp-server/README.md) plugin.

The plugin maps directly to Andrej Karpathy's *Ingest / Query / Lint* primitives, applied to Scraps' typed wiki-link graph:

| Primitive | Component | Role |
| --- | --- | --- |
| Ingest | `ingest` skill | Add a new scrap from a source and update related cross-links |
| Query | `query` skill | Answer wiki questions with citation-rich synthesis |
| Lint | `lint-rule-handler` agent | Purpose-driven wiki health checks, one or a few rules at a time |

## Skills

### `/ingest [source]`

Add a new scrap and integrate it into the existing graph. Sources can be a prompt, a URL, or arbitrary markdown (e.g., a previous query answer for file-back). The skill researches related scraps, drafts the new scrap with appropriate tags and `[[Wiki-links]]`, applies cross-link updates following the link direction protocol (concrete → abstract), and runs a sanity lint.

### `/query [question]`

Answer a question against the wiki. The skill searches scraps with `scraps search`, reads the most relevant ones via `scraps get`, and synthesizes a markdown answer with `[[Title]]` citations. Read-only by design; the user invokes `/ingest` separately if they want to file the answer back.

## Agents

### `lint-rule-handler`

Translates a natural-language wiki-health request (e.g., "fix broken links", "audit orphans") into a small, deliberate set of `scraps lint` rules and runs them. Behavior branches per rule type:

- **Mechanical** (`broken-link`, `broken-heading-ref`, `self-link`) — propose and apply fixes
- **Judgment** (`dead-end`, `lonely`, `overlinking`) — read affected scraps and report; do not auto-fix
- **Informational** (`stale-by-git`) — list with metadata

Lint warnings are signals against a purpose, not absolute errors. The agent rejects "run everything without a purpose" requests and asks for narrowing when intent is vague.

## Integration model

- **Primary path**: `scraps <cmd> --json` invoked via the shell. Works with any agent that can run a shell command, no client implementation needed.
- **Composability**: skills and the agent are independent. `query` does not auto-invoke `ingest`; the user composes them by calling each as needed.
- **Reference docs**: full CLI surface is documented at <https://boykush.github.io/scraps/>. Skill bodies stay focused on workflow and link out for syntax / config detail.

## Further reading

- Scraps documentation site: <https://boykush.github.io/scraps/>
- Karpathy's *LLM Wiki* gist: <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>
