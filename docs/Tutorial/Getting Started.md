This guide gets you from an empty directory to a small Scraps wiki that can be
built as a static site and queried from the CLI.

## Setup

1. **Install Scraps**
   - Follow the [[Tutorial/Installation]] guide to install Scraps on your system

2. **Initialize Project**
   - Create a project directory and initialize it using [[Reference/Init]]:

     ```bash
     ❯ mkdir my-knowledge-base
     ❯ cd my-knowledge-base
     ❯ scraps init
     ```

3. **Configure Project**
   - Edit `.scraps.toml`. The directory containing this file is the wiki root.

## Content Creation

1. **Write Markdown Files**
   - Create Markdown files next to `.scraps.toml` or in folders under it
   - Use [[Reference/CommonMark]] and [[Reference/GitHub-flavored Markdown]]

2. **Add Internal Links**
   - Connect documents using [[Reference/Normal Link]] syntax:
     - `[[Page Name]]` for simple links
     - `[[Page Name|Custom Text]]` for custom link text
     - `[[Folder/Page Name]]` for a context-qualified link

3. **Add Tags**
   - Use [[Reference/Tag Link]] syntax:
     - `#[[Topic]]` for a tag
     - `#[[Area/Subtopic]]` for a nested tag

4. **Enhance Content**
   - Add [[Reference/Mermaid]] diagrams for visual representations
   - Use [[Reference/Autolink]] functionality for external links
   - Organize with [[Reference/Context Link|context folders]] when needed

## Build and Preview

1. **Generate Site**
   - Use [[Reference/Build]] to generate static site files:

     ```bash
     ❯ scraps build
     ```

2. **Preview Locally**
   - Use [[Reference/Serve]] for local preview and debugging:

     ```bash
     ❯ scraps serve
     ```

3. **Deploy**
   - Deploy to platforms like [[How-to/Deploy to GitHub Pages]] when ready

## Quality Check

- **Lint**: Use [[Reference/Lint]] to check wiki-link quality:

  ```bash
  ❯ scraps lint
  ```

## AI Integration

- **CLI JSON**: Query scraps with commands like `scraps search "query" --json`,
  `scraps get "Page Name" --json`, and `scraps todo --json`
- **MCP Server**: Enable MCP-compatible assistant integration using [[How-to/Integrate with AI Assistants]]
