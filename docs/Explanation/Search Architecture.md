#[[Static Site]]

## Search index format

Scraps can build a search index using the Fuse JSON schema as shown below.

```json
[
    {
      "title": "Search",
      "url": "http://127.0.0.1:1112/scraps/search.html"
    },
    {
      "title": "Overview",
      "url": "http://127.0.0.1:1112/scraps/overview.html"
    },
    ...
]
```

## Search libraries

Scraps content perform searches with [fuse.js](https://www.fusejs.io/) using an index.

We are considering WASM solutions like [tinysearch](https://endler.dev/2019/tinysearch) for future performance improvements in our deployment environment.

## Configuration

If you are not using the search function, please modify your `.scraps.toml` as follows. See the [[Configuration]] page for details.

```toml
# Build a search index with the Fuse JSON and display search UI (optional, default=true, choices=true or false)
build_search_index = false
```
