# PKM Plugin

Personal knowledge management skills for a [Scraps](https://github.com/boykush/scraps) wiki.

## Overview

A collection of skills for keeping a wiki as a record of what its owner understands. They run on `scraps <cmd> --json`, like the [`llm-wiki`](../llm-wiki/README.md) plugin, and need no MCP server.

The two plugins differ in whose knowledge the wiki holds. `llm-wiki` follows Andrej Karpathy's *LLM Wiki*, where the model maintains the wiki. `pkm` is for a wiki the user writes for themselves, so its skills keep what the model summarized apart from what the user understood.

## Install

### Step 1: Add the marketplace

```bash
claude plugin marketplace add boykush/scraps
```

### Step 2: Enable the plugin

Add this to your project's `.claude/settings.json`:

```json
{
  "enabledPlugins": {
    "pkm@scraps-claude-code-plugins": true
  }
}
```

The skills operate against the current Scraps wiki. To target a different wiki, set `SCRAPS_DIRECTORY` in your environment.

## Skills

### `/learn [source]`

Learn a URL, article, talk, or term together with the user. The skill explains the source on the pages the wiki already has, checks comprehension with questions, and writes one page from the points that passed. When understanding is not reached, nothing is written.

### `/digest [url]`

Record an article or talk as one short page about that source, linked to existing pages, without dialogue. It never changes existing pages and never claims the user understood the source.

Project instructions (`CLAUDE.md` / `AGENTS.md`) can add local conventions on top, such as a category link or a context folder for source pages.

## Evals

The suite under [`evals/`](evals) runs with [`claude plugin eval`](https://code.claude.com/docs/en/plugin-evals). From the repository root, `mise run plugins:eval` builds scraps and runs it; see [CONTRIBUTING.md](../../CONTRIBUTING.md#plugin-evals).
