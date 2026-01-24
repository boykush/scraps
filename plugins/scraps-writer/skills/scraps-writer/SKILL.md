---
name: scraps-writer
description: Knowledge base for Scraps documentation conventions including Wiki-link syntax, Markdown features, and MCP tools usage.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob
user-invocable: false
argument-hint: [max-lines]
---

# Scraps Writer

Knowledge base for creating Scraps documentation with Wiki-link notation.

## Options

- **max-lines**: `$ARGUMENTS` (optional) - Maximum number of lines for the generated scrap.

## MCP Tools Usage

- `list_tags` → Identify tags for a scrap. Returns all tags sorted by backlinks count.
- `search_scraps` → Find related scraps by keyword with fuzzy matching. Returns `title` and `ctx` for each result.
- `lookup_tag_backlinks` → Check which scraps are using a specific tag.
- `lookup_scrap_links` → Get all scraps that a scrap links to.
- `lookup_scrap_backlinks` → Get all scraps that link to a scrap.

## Wiki-Link Syntax

### Normal Link

`[[Page Name]]` - Links to an existing scrap with the specified title. Use `title` from `search_scraps` results as the Page Name.

### Alias Link

`[[Page Name|Display Text]]` - Links to a scrap with custom display text. Use `title` (and `ctx` if applicable) from `search_scraps` results as the Page Name.

### Context Link

`[[Context/Page Name]]` - Links to a scrap in a specific context folder. Use `ctx` from `search_scraps` results as the Context.

### Tag

`#[[Tag Name]]` - Creates a tag when no scrap with that title exists. Tags group scraps by category and generate an index page listing all scraps using that tag. Use tags from `list_tags` results only.

## Markdown Features

- CommonMark specification
- GitHub-flavored Markdown (tables, task lists, strikethrough)
- Mermaid diagrams with `mermaid` code blocks
- Autolinks for OGP cards: `<https://example.com>`

## File Organization

- Scraps directory is configurable in `.scraps.toml`
- Use folders for context when titles overlap
- Keep folder structure flat (avoid deep nesting)

## Best Practices

- Prefer existing tags over creating new ones
- Keep content focused and concise
- Follow single-responsibility principle for scraps
