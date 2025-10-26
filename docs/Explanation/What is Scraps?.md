![](https://github.com/boykush/scraps/raw/main/assets/logo_opacity.png?raw=true)

Scraps is a portable CLI knowledge hub for managing interconnected Markdown documentation with Wiki-link notation.

## Built for Docs as Code

Scraps is built on the foundation of [Docs as Code](https://www.writethedocs.org/guide/docs-as-code/) principles:

- **Version Control Native**: Documentation evolves with your codebase through Git workflows
- **Review-Driven Quality**: Markdown + Git enables proper documentation review processes
- **Documentation as Code**: Same standards, processes, and tooling as software development
- **Developer Experience**:
  - CLI-first workflow: all functionality accessible through single command-line tool
  - Seamless integration with existing tools (e.g., editors via [[How-to/Setup LSP|LSP]], [[How-to/Deploy to GitHub Pages|GitHub Pages]] deployment, [[How-to/Integrate with AI Assistants|MCP Server]] for AI assistants)

## Core Capabilities

Built on this foundation, Scraps provides three core capabilities:

### 1. Knowledge-First Architecture

Information is organized into atomic, interconnected units following the DRY principle.

- Single source of truth with `[[WikiLink]]` and [[Reference/Context Link|Context]] functionality
- [[Reference/Tag Link|Tags]] for knowledge categorization

### 2. Knowledge Visualization

Transform your knowledge structure into discoverable, navigable experiences.

- Wiki-links become interconnected web experiences
- Static sites with search and pagination for knowledge exploration

### 3. Knowledge Externalization

Make your knowledge accessible beyond traditional documentation boundaries.

- [[How-to/Integrate with AI Assistants|MCP Server]] functionality for AI assistants
- API-like access for external tool integration