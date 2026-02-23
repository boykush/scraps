---
name: add-scrap
description: Create a new scrap with intelligent tag selection and backlink suggestions.
allowed-tools: Read, Glob, WebSearch, Skill
user-invocable: true
argument-hint: "title" [max-lines]
---

# Add Scrap

Create a new scrap with Wiki-link notation.

## Arguments

Parse the following from `$ARGUMENTS`:

- **title** (required, quoted) - Title of the scrap to create. Must be enclosed in double quotes (e.g., `"My Title"`)
- **max-lines** (optional, default: 10) - Maximum number of lines for the generated scrap

If the title is not enclosed in double quotes, ask the user to re-enter it with quotes.

## Workflow

1. **Understand the Request**
   - Ask clarifying questions if the content is unclear

2. **Research the Topic**
   - Use `WebSearch` to gather information about the topic

3. **Create the Scrap**
   - Call `scraps-writer` skill with args: `"<title>" <max-lines>`
