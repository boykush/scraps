## Setup

1. **Install Scraps**
   - Follow the [[Tutorial/Installation]] guide to install Scraps on your system

2. **Initialize Project**
   - Create a new Scraps project using [[Reference/Init]]:
     ```bash
     ❯ scraps init my-knowledge-base
     ❯ cd my-knowledge-base
     ```

3. **Configure Project**
   - Follow [[Tutorial/Configure Project]] to set up your `Config.toml`

## Content Creation

1. **Write Markdown Files**
   - Create Markdown files in the `/scraps` directory
   - Use [[Reference/CommonMark]] and [[Reference/GitHub-flavored Markdown]]

2. **Add Internal Links**
   - Connect documents using [[Reference/Normal Link]] syntax:
     - `[[Page Name]]` for simple links
     - `[[Page Name|Custom Text]]` for custom link text

3. **Enhance Content**
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

## AI Integration

- **MCP Server**: Enable AI assistant integration using [[How-to/Integrate with AI Assistants]] for intelligent search and content assistance
