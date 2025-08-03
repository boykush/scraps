## Setup

1. **Install Scraps**
   - Follow the [[Installation]] guide to install Scraps on your system

2. **Initialize Project**
   - Create a new Scraps project using [[CLI/Init]]:
     ```bash
     ❯ scraps init my-knowledge-base
     ❯ cd my-knowledge-base
     ```

3. **Configure Project**
   - Customize the [[Configuration]] in `Config.toml`
   - Set your site title, base URL, and other preferences

## Content Creation

1. **Write Markdown Files**
   - Create Markdown files in the `/scraps` directory
   - Use [[CommonMark specification]] and [[GitHub-flavored Markdown]]

2. **Add Internal Links**
   - Connect documents using [[Internal Link]] syntax:
     - `[[Page Name]]` for simple links
     - `[[Page Name|Custom Text]]` for custom link text

3. **Enhance Content**
   - Add [[Mermaid]] diagrams for visual representations
   - Use [[Autolink]] functionality for external links
   - Organize with [[Context link|context folders]] when needed

## Build and Preview

1. **Generate Site**
   - Use [[CLI/Build]] to generate static site files:
     ```bash
     ❯ scraps build
     ```

2. **Preview Locally**
   - Use [[CLI/Serve]] for local preview and debugging:
     ```bash
     ❯ scraps serve
     ```

3. **Deploy**
   - Deploy to platforms like [[GitHub Pages]] when ready
