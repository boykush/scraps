---
name: scraps-writer
description: Shared workflow for creating scraps with tag research, Wiki-link resolution, and content verification.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob
user-invocable: false
argument-hint: "title" [max-lines]
---

# Scraps Writer

Shared workflow for creating scraps with Wiki-link notation.

## Arguments

Parse the following from `$ARGUMENTS`:

- **title** (required, quoted) - Title of the scrap to create. Must be enclosed in double quotes (e.g., `"My Title"`)
- **max-lines** (optional, default: 10) - Maximum number of lines for the generated scrap

Parse the title by extracting the text between the first pair of double quotes. Everything after the closing quote is parsed as remaining arguments.

## Workflow

1. **Research Existing Tags** (snapshot for verification)
   - Use `list_tags` to get available tags
   - Save this list for the verification step later
   - Identify tags relevant to the topic

2. **Search Related Scraps**
   - Use `search_scraps` to find related scraps (returns `title` and `ctx` only)
   - Use `get_scrap` to retrieve full content of specific scraps when needed
   - Identify scraps that should link to the new scrap
   - Check if a scrap with the same **title** already exists. If so, determine an appropriate context folder name to disambiguate

3. **Create the Scrap**
   - Write well-structured Markdown content following the syntax below
   - If summarizing a web article, include the source URL as autolink: `<https://...>`
   - Filename: `scraps/<title>.md`, or `scraps/<ctx>/<title>.md` if a context folder is needed to avoid title conflicts
   - **IMPORTANT**: The scrap content MUST NOT exceed **max-lines** lines. Count the total lines before writing and trim if necessary

4. **Verify Tag Consistency**
   - Use `list_tags` again and compare with the result from step 1
   - If new tags appeared, find the `[[...]]` links that caused them and remove the `[[]]` notation (leave as plain text)

5. **Suggest Backlinks**
   - List existing scraps that should add links to this new scrap

## MCP Tools Usage

- `list_tags` - Get all tags sorted by backlinks count.
- `search_scraps` - Find related scraps by keyword with fuzzy matching against title and body content. Returns `title` and `ctx` for each result. **IMPORTANT: Each result represents an existing scrap. Only use the returned `title` and `ctx` values when creating Wiki-links.**
- `get_scrap` - Get a single scrap's full content by `title` and optional `ctx`.
- `lookup_tag_backlinks` - Check which scraps are using a specific tag. Returns `title` and `ctx` for each result.
- `lookup_scrap_links` - Get all scraps that a scrap links to. Returns `title` and `ctx` for each result.
- `lookup_scrap_backlinks` - Get all scraps that link to a scrap. Returns `title` and `ctx` for each result.

## Wiki-Link Syntax

- `[[Page Name]]` - Normal link. Use `title` from `search_scraps` results.
- `[[Page Name|Display Text]]` - Alias link with custom display text.
- `[[Context/Page Name]]` - Context link. Use `ctx/title` from `search_scraps` results.
- `#[[Tag Name]]` - Tag. Use tags from `list_tags` results only.

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
