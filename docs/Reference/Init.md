#[[CLI]]

```bash
❯ scraps init
```

This command initializes the current directory as a Scraps wiki. Create the
directory first, then run `scraps init` inside it.

```bash
❯ tree -a -L 1
.
├── .gitignore    # Git ignore patterns for Scraps projects
└── .scraps.toml  # Project configuration file
```

## Examples

```bash
# Initialize new project
❯ mkdir my-knowledge-base
❯ cd my-knowledge-base
❯ scraps init

# Initialize a specific existing directory
❯ scraps -C /path/to/wiki init
```

After initializing the project, proceed to [[Reference/Build|Build]] to generate your static site.
