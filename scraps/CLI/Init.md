#[[CLI]]

```bash
❯ scraps init your-scraps-project
❯ cd your-scraps-project
```

This command initializes a new Scraps project. It creates the following structure:

```bash
❯ tree -a -L 1
.
├── .gitignore    # Git ignore patterns for Scraps projects
├── Config.toml   # Project configuration file
└── scraps       # Directory for your Markdown files
```

After initializing the project, proceed to [[CLI/Build|Build]] to generate your static site.