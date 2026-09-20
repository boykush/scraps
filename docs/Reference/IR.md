The Scraps IR is what the compiler knows about a wiki after reading it: one
object per Markdown source, and the link table produced by resolving every
reference against those objects. Every command reads the wiki through it,
and `scraps build` is one of its emitters, so the [[Reference/Static Site]]
is an output of the IR rather than the only artifact Scraps keeps.

## Location

The IR lives in `.scraps/ir.json` next to `.scraps.toml`, the way `target/`
sits next to `Cargo.toml`:

```text
.scraps.toml
.scraps/
├── .gitignore      # "*" — the directory ignores itself
├── CACHEDIR.TAG    # backup tools skip it
└── ir.json
```

The directory is derived and disposable. It is never committed, needs no
entry in the wiki's own `.gitignore`, and deleting it only costs one full
parse on the next command.

## Lifecycle

Any command — `build`, `get`, `search`, `lint`, an MCP tool — loads the IR
the same way:

1. Walk the wiki and hash every Markdown source.
2. A source whose hash the IR already holds is rebuilt from its stored
   object without parsing.
3. Every other source is parsed, and objects for removed sources are
   dropped.
4. If anything changed, the link table is resolved again and the file is
   rewritten in place.

Freshness is decided by content hash, not modification time, so a `git
checkout` that rewrites timestamps invalidates nothing. A file written by
another Scraps release or format version is discarded and rebuilt, never
migrated. A corrupt or unwritable file is ignored rather than failing the
command.

## Contents

```json
{
  "format_version": 1,
  "scraps_version": "2.3.0",
  "scraps": [
    {
      "path": "Reference/Static Site/Pipeline.md",
      "hash": "201b925e42f4fe4b",
      "title": "Pipeline",
      "ctx": "Reference/Static Site",
      "refs": [
        { "kind": "tag", "tag": "Emit/Static Site", "line": 1 },
        { "kind": "link", "title": "Deploy to GitHub Pages", "ctx": "How-to",
          "heading": null, "alias": null, "line": 12 }
      ],
      "headings": [],
      "code_blocks": [],
      "images": [],
      "task_items": [],
      "frontmatter": null
    }
  ],
  "links": [
    { "from": { "title": "Pipeline", "ctx": "Reference/Static Site" },
      "to": { "title": "Deploy to GitHub Pages", "ctx": "How-to" },
      "kind": "link", "heading": null, "line": 12, "resolved": true }
  ]
}
```

- `scraps` holds one object per source file, in path order: its identity
  (`path`, `title`, `ctx`), the `hash` of its content, and everything the
  parser found — `refs` (every `[[link]]`, `![[embed]]` and `#[[tag]]`
  occurrence with its line), `headings`, `code_blocks`, `images`,
  `task_items`, and the passthrough `frontmatter`.
- `links` is the link table: one edge per link or embed occurrence, with
  `resolved: false` when the target names no scrap. Tags are attributes of
  the object, not edges, matching the `[[ ]]` / `#[[ ]]` namespace split.
- `commited_ts` on an object and `git_head` at the top appear once a
  build has read git, which it does unless `--no-git`: the last-commit
  timestamp of each source and the HEAD it was read under. While HEAD is
  unchanged the next build reuses them instead of reading the history again.
- Bodies are not stored. The Markdown files stay the only source of truth;
  the IR is a read model derived from them.

The shape is versioned by `format_version` and may change between Scraps
releases. The stable contract for scripts and agents remains the `--json`
output in [[Reference/CLI Overview]]; read `ir.json` directly only when a
one-off script wants the whole graph at once.
