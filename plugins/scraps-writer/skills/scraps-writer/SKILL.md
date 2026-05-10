---
name: scraps-writer
description: Shared workflow for creating scraps with tag research, Wiki-link resolution, and content verification.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob, Bash
user-invocable: true
argument-hint: "title" [max-lines]
---

# Scraps Writer

Shared workflow for creating scraps with Wiki-link notation.

## Arguments

Parse the following from `$ARGUMENTS`:

- **title** (required, quoted) - Title of the scrap to create. Must be enclosed in double quotes (e.g., `"My Title"`)
- **max-lines** (optional) - Maximum number of lines for the generated scrap. If omitted, automatically determined based on topic familiarity (see step 3)

Parse the title by extracting the text between the first pair of double quotes. Everything after the closing quote is parsed as remaining arguments.

## Workflow

1. **Research Existing Tags**
   - Use `list_tags` to get available tags
   - Identify tags relevant to the topic

2. **Search Related Scraps**
   - Use `search_scraps` to find related scraps (returns `title` and `ctx` only)
   - Use `get_scrap` to retrieve full content of specific scraps when needed
   - Identify scraps that should link to the new scrap
   - Check if a scrap with the same **title** already exists. If so, determine an appropriate context folder name to disambiguate
   - If a related scrap covers a similar topic, consider how the new scrap adds distinct value — focus on a different aspect, a more specific subtopic, or a different perspective rather than duplicating existing content

3. **Estimate Topic Familiarity** (only when max-lines is not explicitly provided)
   - Use the `count` from `search_scraps` in step 2 — this directly reflects how much the user has written about closely related subjects
   - Determine familiarity level (inverted-U curve — cognitive load theory + expertise reversal effect):
     - **Low familiarity** (0–5 related scraps): set max-lines to **5–7**. Lacking schema, too much information overwhelms working memory (cognitive load theory, Sweller)
     - **Medium familiarity** (6–15 related scraps): set max-lines to **10–12**. Sufficient prior knowledge to process detailed content effectively (schema theory)
     - **High familiarity** (16+ related scraps): set max-lines to **5–7**. Redundant explanations become counterproductive; concise content with links is more effective (expertise reversal effect, Kalyuga & Sweller)
   - Report the chosen max-lines and the reasoning (e.g., "12 related scraps found → high familiarity → 18 lines")

4. **Create the Scrap**
   - Write well-structured Markdown content following the syntax below
   - If summarizing a web article, include the source URL as autolink: `<https://...>`
   - Filename: `scraps/<title>.md`, or `scraps/<ctx>/<title>.md` if a context folder is needed to avoid title conflicts
   - **Line limit**: The scrap content must not exceed **max-lines** lines. Scraps are designed as concise, focused knowledge units — keeping them short makes the wiki scannable and encourages linking between scraps rather than cramming everything into one page. Count the total lines before writing and trim if necessary

5. **Lint Tag Quality**
   - Run `scraps lint --rule singleton-tag` command via Bash tool from the project root directory
   - Check the output for `singleton-tag` warnings — these indicate tags referenced by only 1 scrap
   - If any singleton-tag warnings are caused by the newly created scrap, fix them by removing the `[[]]` notation from the offending tag links (leave as plain text)

6. **Suggest Backlinks**
   - List existing scraps that should add links to this new scrap
   - Good candidates: scraps that share the same tags, cover a parent/sibling topic, or mention concepts that the new scrap explains in more detail

## MCP Tools Usage

- `list_tags` - Get all tags sorted by backlinks count.
- `search_scraps` - Find related scraps by keyword with fuzzy matching against title and body content. Returns `title` and `ctx` for each result. **IMPORTANT: Each result represents an existing scrap. Only use the returned `title` and `ctx` values when creating Wiki-links.**
- `get_scrap` - Get a single scrap by `title`, optional `ctx`, optional `heading`, and optional `fields`. Defaults to `title`, `ctx`, and `body`.
- `lookup_tag_backlinks` - Check which scraps are using a specific tag. Returns `title` and `ctx` for each result.
- `lookup_scrap_links` - Get outbound link/embed refs from a scrap. Returns `kind`, `title`, `ctx`, and `heading` for each result.
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
