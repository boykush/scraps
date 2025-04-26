#[[CLI]]

```bash
❯ scraps build
```

This command processes Markdown files from the `/scraps` directory and generates a static website.

#### Source Structure
```bash
❯ tree scraps
scraps
├── Getting Started.md
└── What is Scraps?.md
```

#### Generated Files
The command generates the following files in the `public` directory:
```bash
❯ tree public
public
├── index.html      # Main page with scrap list
├── getting-started.html
├── what-is-scraps.html
├── main.css       # Styling for the site
└── search.json    # Search index (if enabled)
```

Each Markdown file is converted to a slugified HTML file. Additional files like `index.html` and `main.css` are generated to create a complete static website.

After building, use [[CLI/Serve]] to preview your site locally.