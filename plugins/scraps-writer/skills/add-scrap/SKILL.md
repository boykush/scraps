---
name: add-scrap
description: Create a new scrap with intelligent tag selection and backlink suggestions. Use this skill whenever the user wants to create a new scrap, write about a topic, add documentation, or says something like "add a scrap about X" or "write about X".
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
   - Consider asking about: scope of the topic, target audience, related technologies or concepts, and whether the user has specific aspects they want to focus on

2. **Research the Topic**
   - Use `WebSearch` to gather information about the topic
   - Focus on: official documentation, technical definitions, recent developments, and key concepts that help write an accurate and concise scrap

3. **Create the Scrap**
   - Call `scraps-writer` skill with args: `"<title>" <max-lines>`
