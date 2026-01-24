---
name: add-scrap
description: Create a new scrap with intelligent tag selection and backlink suggestions.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob, WebSearch, Skill
user-invocable: true
argument-hint: [title] [max-lines]
---

# Add Scrap

Create a new scrap by calling `scraps-writer` skill.

## Arguments

- **title**: `$ARGUMENTS` - Title of the scrap to create
- **max-lines**: (optional) - Maximum number of lines for the generated scrap

## Workflow

1. **Understand the Request**
   - Ask clarifying questions if the content is unclear
   - Identify the context folder if applicable

2. **Research the Topic**
   - Use `WebSearch` to gather information about the topic

3. **Research Existing Tags**
   - Use `list_tags` to get available tags
   - Identify tags relevant to the topic

4. **Search Related Scraps**
   - Use `search_scraps` to find related content
   - Identify scraps that should link to the new scrap

5. **Create the Scrap**
   - Call `scraps-writer` skill with max-lines argument
   - Write well-structured Markdown content

6. **Suggest Backlinks**
   - List existing scraps that should add links to this new scrap
