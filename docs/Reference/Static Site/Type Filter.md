---
type: Reference
---

#[[Emit/Static Site]]

When at least one scrap carries the frontmatter `type` key described in
[[Reference/Frontmatter]], every list view gains a type filter: a row above
the list naming each type in use with its scrap count, most used first.
Choosing a type narrows that same view to the scraps of that type, the way a
label narrows an issue list; `all` returns to the whole view.

A scrap's own page names its type under the title and links to the updated
view narrowed to it. An untyped scrap's page shows nothing.

A wiki in which no scrap has a type gets none of this: no filter and no
`types/` directory.

## Paths

A narrowed view is a URL, like the views themselves in
[[Reference/Static Site/Sort and Pagination]]. Each type gets the three views
again under `types/<type>/`, and the sidebar's views keep the filter, so
changing the sort order does not drop it.

| View | Whole wiki | One type |
|---|---|---|
| updated | `/` | `/types/<type>/` |
| backlinks | `/backlinks/` | `/types/<type>/backlinks/` |
| all scraps | `/all-scraps/` | `/types/<type>/all-scraps/` |

The narrowed updated and backlinks views paginate by `paginate_by` like the
whole ones.

The directory is the type as written: `BigQuery Table` is
`types/BigQuery Table/`. Types compare exactly, case included, so `Blog` and
`blog` are two types with two directories. A type has no hierarchy, so a `/`
in its name is written `%2F` instead of becoming a directory; the naming rule
is the one in [[Reference/Static Site/Output Structure]].

| Type | Path |
|---|---|
| `Blog` | `types/Blog/` |
| `blog` | `types/blog/` |
| `ADR/Accepted` | `types/ADR%2FAccepted/` |
