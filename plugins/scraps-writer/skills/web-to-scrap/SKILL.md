---
name: web-to-scrap
description: Summarize a web article and create a scrap with source link and tags.
allowed-tools: mcp__plugin_scraps-writer_scraps__*, Read, Write, Edit, Glob, WebFetch, Skill
user-invocable: true
argument-hint: [url] [max-lines]
---

# Web to Scrap

Summarize a web article and create a scrap by calling `scraps-writer` skill.

## Arguments

- **url**: `$ARGUMENTS` - URL of the web article to summarize
- **max-lines**: (optional) - Maximum number of lines for the generated scrap

## Workflow

1. **Fetch the Article**
   - Use `WebFetch` to retrieve the web article content
   - Extract the title and main content

2. **Research Existing Tags**
   - Use `list_tags` to get available tags
   - Identify tags relevant to the article topic

3. **Search Related Scraps**
   - Use `search_scraps` to find related content

4. **Create the Scrap**
   - Call `scraps-writer` skill with max-lines argument
   - Generate a concise summary of the article
   - Include the source URL as autolink: `<https://...>`
