#[[Emit/Static Site]]

Build output is written to the directory configured by `output_dir`
(default `_site/`).

```bash
❯ tree _site
_site
├── index.html              # home: the updated view (plus README.md, when present)
├── backlinks/
│   └── index.html          # most linked first
├── titles/
│   └── index.html          # title index, grouped by initial
├── scraps/
│   ├── index.html          # every scrap, in title order
│   ├── getting-started.html
│   └── guide/
│       └── links.html
├── main.css
└── search_index.json       # when build_search_index = true
```

`scraps/index.html` lists every scrap in title order on a single page, with
no pagination, so a wiki can be browsed whole however large it grows. The
root index and `backlinks/` stay paginated; see
[[Reference/Static Site/Sort and Pagination]].

Each Markdown file is converted to a slugified HTML file under `scraps/`.
Folders become path segments — the same folders that form
[[Reference/Wiki-link/Context Link]].

Files in `static/` and the build output directory are excluded from scrap
traversal.
