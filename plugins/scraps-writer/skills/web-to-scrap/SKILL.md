---
name: web-to-scrap
description: Summarize a web article and create a scrap with source link and tags.
allowed-tools: Read, Glob, WebFetch, Skill
user-invocable: true
argument-hint: [url] [max-lines]
---

# Web to Scrap

Summarize a web article and create a scrap with Wiki-link notation.

## Arguments

- **url**: `$ARGUMENTS` - URL of the web article to summarize
- **max-lines**: (optional, default: 10) - Maximum number of lines for the generated scrap

## Workflow

1. **Fetch the Article**
   - Use `WebFetch` to retrieve the web article content
   - Extract the OGP title and use it as the scrap title

2. **Create the Scrap**
   - Call `scraps-writer` skill with max-lines argument
   - The scrap should be a concise summary of the article with a source autolink
