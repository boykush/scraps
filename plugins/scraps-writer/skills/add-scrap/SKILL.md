---
name: add-scrap
description: Create a new scrap with intelligent tag selection and backlink suggestions.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob, WebSearch
user-invocable: true
argument-hint: [title] [max-lines]
---

# Add Scrap

Create a new scrap with Wiki-link notation.

## Arguments

- **title**: `$ARGUMENTS` - Title of the scrap to create
- **max-lines**: (optional, default: 10) - Maximum number of lines for the generated scrap

## Workflow

1. **Understand the Request**
   - Ask clarifying questions if the content is unclear
   - Identify the context folder if applicable

2. **Research the Topic**
   - Use `WebSearch` to gather information about the topic

3. **Research Existing Tags** (snapshot for verification)
   - Use `list_tags` to get available tags
   - Save this list for the verification step later
   - Identify tags relevant to the topic

4. **Search Related Scraps**
   - Use `search_scraps` to find related content
   - Identify scraps that should link to the new scrap

5. **Create the Scrap**
   - Write well-structured Markdown content following the syntax below
   - If `max-lines` is specified, keep the scrap within that line count

6. **Verify Tag Consistency**
   - Use `list_tags` again and compare with the result from step 3
   - If new tags appeared, find the `[[...]]` links that caused them and remove the `[[]]` notation (leave as plain text)

7. **Suggest Backlinks**
   - List existing scraps that should add links to this new scrap

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
