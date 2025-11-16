#[[CLI]]

```bash
❯ scraps init <PROJECT_NAME>
```

This command initializes a new Scraps project. It creates the following structure:

```bash
❯ tree -a -L 1
.
├── .gitignore    # Git ignore patterns for Scraps projects
├── Config.toml   # Project configuration file
└── scraps       # Directory for your Markdown files
```

## Examples

```bash
# Initialize new project
❯ scraps init my-knowledge-base
❯ cd my-knowledge-base

# Initialize with specific path
❯ scraps init docs --path /path/to/workspace
```

After initializing the project, proceed to [[Reference/Build|Build]] to generate your static site.
