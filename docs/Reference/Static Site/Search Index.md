#[[Emit/Static Site]]

When `build_search_index = true` (the default), the build emits
`search_index.json` in the [Fuse.js](https://www.fusejs.io/) JSON shape and
the static site mounts a search UI backed by it.

```json
[
  { "title": "Getting Started", "url": "https://example.com/scraps/getting-started.html" },
  { "title": "Configuration",   "url": "https://example.com/scraps/configuration.html" }
]
```

Search is **scoped to the static site** — it powers the in-page search box
for human readers. Agent-side search is `scraps search --json` and is
independent of this index.

Set `build_search_index = false` in [[Reference/Configuration]] to skip
both the index and the UI.
