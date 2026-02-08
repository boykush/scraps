---
name: web-to-scrap
description: Summarize a web article and create a scrap with source link and tags.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob, WebFetch
user-invocable: true
argument-hint: [url] [max-lines]
---

# Web to Scrap

Summarize a web article and create a scrap with Wiki-link notation.

## Arguments

- **url**: `$ARGUMENTS` - URL of the web article to summarize
- **max-lines**: (optional) - Maximum number of lines for the generated scrap

## Workflow

1. **Fetch the Article**
   - Use `WebFetch` to retrieve the web article content
   - Extract the title and main content

2. **Research Existing Tags** (snapshot for verification)
   - Use `list_tags` to get available tags
   - Save this list for the verification step later
   - Identify tags relevant to the article topic

3. **Search Related Scraps**
   - Use `search_scraps` to find related content

4. **Create the Scrap**
   - Generate a concise summary of the article
   - Include the source URL as autolink: `<https://...>`
   - Write well-structured Markdown content following the syntax below

5. **Verify Tag Consistency**
   - Use `list_tags` again and compare with the result from step 2
   - If new tags appeared, find the `[[...]]` links that caused them and remove the `[[]]` notation (leave as plain text)

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
