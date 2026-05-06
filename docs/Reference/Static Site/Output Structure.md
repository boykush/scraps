#[[Emit/Static Site]]

Build output is written to the directory configured by `output_dir`
(default `_site/`).

```bash
❯ tree _site
_site
├── index.html              # README.md or scrap index
├── scraps/
│   ├── getting-started.html
│   └── guide/
│       └── links.html
├── main.css
└── search_index.json       # when build_search_index = true
```

Each Markdown file is converted to a slugified HTML file under `scraps/`.
Folders become path segments — the same folders that form
[[Reference/Wiki-link/Context Link]].

Files in `static/` and the build output directory are excluded from scrap
traversal.
