---
type: Reference
---

#[[Emit/Static Site]]

Build output is written to the directory configured by `output_dir`
(default `_site/`).

```bash
❯ tree _site
_site
├── index.html              # home: the updated view
├── about/
│   └── index.html          # README.md, when present
├── backlinks/
│   └── index.html          # most linked first
├── all-scraps/
│   └── index.html          # every scrap, grouped by the title's initial
├── all-tags/
│   └── index.html          # every tag, most used first
├── scraps/                 # one page per scrap, nothing else
│   ├── Getting Started.html
│   └── Guide/
│       └── Links.html
├── tags/                   # one page per tag, nothing else
│   ├── Markdown.html
│   └── Notation/
│       └── Wiki-link.html
├── types/                  # only when a scrap has a type
│   └── How-to/             # the views again, narrowed to one type
│       ├── index.html
│       ├── backlinks/
│       │   └── index.html
│       └── all-scraps/
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

`all-scraps/index.html` lists every scrap on a single page, grouped by the
title's initial with a jump bar and no pagination, so a wiki can be browsed
whole however large it grows. The root index and `backlinks/` stay
paginated; see [[Reference/Static Site/Sort and Pagination]].

## Page paths

A page's path is its name as written. A scrap's name is its key: the folders
that form its [[Reference/Wiki-link/Context Link]] and the file name without
`.md`, with case, spaces and symbols kept. `Reference/CLI Overview.md` is
written to `scraps/Reference/CLI Overview.html`.

Nothing is slugified, so two Markdown files the filesystem keeps apart are two
pages: `a b.md` and `a-b.md` never share one. A tidy URL comes from a tidy
file name; the keys a scrap can set for itself are in
[[Reference/Frontmatter]].

Names are compared in Unicode NFC, so the name of a scrap, a tag or a type
matches however the filesystem or an editor composed it, and its page path is
the same on every machine. Content is left as written. Two files whose names
differ only in normalization form are one name, so a wiki holding both does
not load.

A link to a page percent-encodes each path segment, so the page above is
served at `scraps/Reference/CLI%20Overview.html`.

Tags and types follow the same rule in directories of their own, so a scrap,
a tag and a type of the same name never share a path; see
[[Reference/Static Site/Tag Pages]] and [[Reference/Static Site/Type Filter]].
A tag segment or a type is free text rather than a file name, so three
spellings are escaped in the file name to keep every name on its own path
inside its directory:

| In the name | In the file name |
|---|---|
| `%` | `%25` |
| `/` in a type | `%2F` |
| a tag segment or type that is exactly `.` or `..` | `%2E` or `%2E%2E` |

`scraps/` and `tags/` hold only pages named by a scrap or a tag, and a type's
directory is named by the type, so no name can land on a listing: a scrap or
a tag called `index` is a page like any other.

Files in `static/`, the build output directory, and any directory whose name
starts with a dot (such as `.git/` or the `.scraps/` [[Reference/IR]]) are
excluded from scrap traversal.
