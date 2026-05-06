#[[CLI]]

```bash
❯ scraps build
```

This command processes Markdown files under the wiki root and generates a static website. The wiki root is the directory containing `.scraps.toml`.

## Source Structure

```bash
❯ tree -a -I _site
.
├── .scraps.toml
├── Getting Started.md
├── Documentation.md
└── Guide
    └── Links.md
```

## Generated Files

The command generates the following files in the `_site` directory (configurable via `output_dir` in `.scraps.toml`):

```bash
❯ tree _site
_site
├── index.html      # Main page with scrap list
├── scraps
│   ├── getting-started.html
│   ├── documentation.html
│   └── guide
│       └── links.html
├── main.css       # Styling for the site
└── search_index.json # Search index (if enabled)
```

Each Markdown file is converted to a slugified HTML file. `README.md` is special:
it becomes the site top page (`index.html`) instead of a normal scrap page.
Files in `static/` and the build output directory are excluded from scrap
traversal.

## Examples

```bash
# Basic build
❯ scraps build

# Build with verbose output
❯ scraps build --verbose

# Include git-derived committed timestamps in generated pages
❯ scraps build --git

# Build from specific directory
❯ scraps -C /path/to/wiki build
```

After building, use [[Reference/Serve]] to preview your site locally.
