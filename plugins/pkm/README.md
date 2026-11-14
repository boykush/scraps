# PKM Plugin

Personal knowledge management skills for a [Scraps](https://github.com/boykush/scraps) wiki.

## Overview

A collection of skills for keeping a wiki as a record of what its owner understands, and for keeping that record true as time passes. They run on `scraps <cmd> --json`, like the [`llm-wiki`](../llm-wiki/README.md) plugin, and need no MCP server.

The two plugins differ in whose knowledge the wiki holds. `llm-wiki` follows Andrej Karpathy's *LLM Wiki*, where the model maintains the wiki. `pkm` is for a wiki the user writes for themselves, so its skills keep what the model summarized apart from what the user understood, and record when the user confirmed a page is still current.

## Who stands behind a page

Every skill records it in the page's `verified` frontmatter, and the actor says which of two things was confirmed:

| | An agent stands behind it | The user stands behind it |
|---|---|---|
| Says | the page was checked against its sources | the user understands what the page says |
| Written by | `digest`, `learn`, `verify` | `learn`, `recall`, `verify` on the user's word |
| `by` | `<producer>/<version>` | `human:<id>` |
| Tier | `machine-confirmed` | `human-reviewed` |
| Grows old | `scraps verified --by machine`, renewed by `verify` | `scraps verified --by human`, renewed by `recall` |

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

### `/verify [page]`

Re-check an existing page against today's primary sources together with the user. Changed claims are explained and checked the way `learn` checks a new page, and only what the user confirmed is rewritten. A page that checks out gets the agent's own `verified` entry, and the user's too when they confirm the page as a whole, which makes it `human-reviewed`. With no page named, it takes stock of the wiki and proposes the pages most likely to have drifted.

### `/recall [key]`

Check again that the user still recalls a page they confirmed. Where `/verify` asks whether the page is still true to the world, `/recall` asks whether it is still true to the user. The skill takes the pages a person confirmed longest ago (`scraps verified --by human`), asks the user to recall each one before showing it, and renews the confirmation of those that passed. A page that did not pass is left as it is and stays due.

### `/digest [url]`

Record an article or talk as one short page about that source, linked to existing pages, without dialogue. It never changes existing pages and never claims the user understood the source.

Project instructions (`CLAUDE.md` / `AGENTS.md`) can add local conventions on top, such as a category link or a context folder for source pages.

## Evals

The suite under [`evals/`](evals) runs with [`claude plugin eval`](https://code.claude.com/docs/en/plugin-evals). From the repository root, `mise run plugins:eval` builds scraps and runs it; see [CONTRIBUTING.md](../../CONTRIBUTING.md#plugin-evals).
