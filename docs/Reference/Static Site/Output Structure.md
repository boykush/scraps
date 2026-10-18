---
type: Reference
---

#[[Emit/Static Site]]

Build output is written to the directory configured by `output_dir`
(default `_site/`).

```bash
❯ tree _site
_site
├── index.html              # home: the list of every scrap
├── about/
│   └── index.html          # README.md, when present
├── all-tags/
│   └── index.html          # every tag, most used first
├── scraps/                 # one page per scrap, nothing else
│   ├── Getting Started.html
│   └── Guide/
│       └── Links.html
├── tags/                   # one list per tag, nothing else
│   ├── Markdown/
│   │   └── index.html
│   └── Notation/
│       ├── index.html
│       └── Wiki-link/
│           └── index.html
├── fonts/                  # the web fonts the theme uses, with their licenses
│   ├── fonts.css
│   └── *.woff2
├── js/                     # the libraries the pages load, with their licenses
│   ├── fuse.min.js
│   ├── highlight.min.js
│   └── mermaid.min.js
├── main.css
└── search_index.json       # when build_search_index = true
```

Sort order and type are query parameters on these lists rather than pages of
their own; see [[Reference/Static Site/Sort and Pagination]] and
[[Reference/Static Site/Type Filter]].

## Page paths

A page's path is its name as written. A scrap's page is named by its key
([[Reference/Wiki-link Notation#Key, title and context]]), with case, spaces
and symbols kept: `Reference/CLI Overview.md` is written to
`scraps/Reference/CLI Overview.html`. The title never enters a path.

Nothing is slugified, so two Markdown files the filesystem keeps apart are two
pages: `a b.md` and `a-b.md` never share one. A tidy URL comes from a tidy
file name, and a frontmatter `title` keeps the free-text name on the page:
`tables/orders.md` with `title: Customer Orders` is served at
`scraps/tables/orders.html` and headed "Customer Orders".

Names are compared in Unicode NFC, so the name of a scrap, a tag or a type
matches however the filesystem or an editor composed it, and its page path is
the same on every machine. Content is left as written. Two files whose names
differ only in normalization form are one name, so a wiki holding both does
not load.

A link to a page percent-encodes each path segment, so the page above is
served at `scraps/Reference/CLI%20Overview.html`.

Tags follow the same rule in a directory of their own, so a scrap and a tag
of the same name never share a path; see [[Reference/Static Site/Tag Pages]].
A tag segment is free text rather than a file name, so a few spellings are
escaped in the path to keep every tag on its own path inside its directory:

| In the tag segment | In the path |
|---|---|
| `%` | `%25` |
| exactly `.` or `..` | `%2E` or `%2E%2E` |
| ending in `.html` | that `.` as `%2E` |

A child tag's directory sits beside its parent's `index.html`, which is why a
segment ending in `.html` is escaped. `scraps/` and `tags/` hold only pages
named by a scrap or a tag, so no name can land on a listing: a scrap or a tag
called `index` is a page like any other.

Files in `static/`, the build output directory, and any directory whose name
starts with a dot (such as `.git/` or the `.scraps/` [[Reference/IR]]) are
excluded from scrap traversal.
