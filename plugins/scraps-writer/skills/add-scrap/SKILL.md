---
name: add-scrap
description: Create a new scrap with intelligent tag selection and backlink suggestions.
allowed-tools: Read, Glob, WebSearch, Skill
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

3. **Create the Scrap**
   - Call `scraps-writer` skill with max-lines argument
