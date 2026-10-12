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
again under `types/<slug>/`, and the sidebar's views keep the filter, so
changing the sort order does not drop it.

| View | Whole wiki | One type |
|---|---|---|
| updated | `/` | `/types/<slug>/` |
| backlinks | `/backlinks/` | `/types/<slug>/backlinks/` |
| all scraps | `/scraps/` | `/types/<slug>/scraps/` |

The narrowed updated and backlinks views paginate by `paginate_by` like the
whole ones.

The slug is the type name slugified as one segment: `BigQuery Table` becomes
`types/bigquery-table/`. A type has no hierarchy, so a `/` in its name does
not become a directory.

Types are case-sensitive and slugs are not, so two types can ask for the same
path. Neither overwrites the other: in name order, the first keeps the slug
and each later one takes the next free numeric suffix.

| Type | Path |
|---|---|
| `Blog` | `types/blog/` |
| `blog` | `types/blog-2/` |
