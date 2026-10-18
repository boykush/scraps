---
type: Reference
---

#[[Emit/Static Site]]

When at least one scrap carries the frontmatter `type` key described in
[[Reference/Frontmatter]], every list gains a type row: each type in use with
the number of the list's scraps of that type, most used across the wiki
first. Choosing a type narrows that same list to the scraps of that type, the
way a label narrows an issue list; `all` returns to the whole list. A type the
list holds none of is shown but cannot be chosen.

The type is the `type` query parameter, next to `sort` from
[[Reference/Static Site/Sort and Pagination]]: `/tags/Rust/?type=Book&sort=backlinks`
is the Book scraps tagged Rust, most linked first. Switching to another tag in
the sidebar keeps both, and the sidebar counts each tag's scraps of the chosen
type, so a tag holding none of it cannot be chosen either.

Types compare exactly, case included, so `Blog` and `blog` are two types.

A scrap's own page names its type under the title and links to the whole
wiki's list narrowed to it. An untyped scrap's page shows nothing.

A wiki in which no scrap has a type gets no type row.
