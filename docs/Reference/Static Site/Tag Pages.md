---
type: Reference
---

#[[Emit/Static Site]]

Each tag carried by at least one scrap, as `#[[tag]]` in its body or in its
frontmatter `tags` list, gets its own list of the scraps that carry it.
Nested tags (`#[[a/b/c]]`) auto-aggregate to their parents — see
[[Reference/Wiki-link/Nested Tag]]. A tag's list is ordered and narrowed like
the whole wiki's: see [[Reference/Static Site/Sort and Pagination]] and
[[Reference/Static Site/Type Filter]].

The sidebar lists the whole wiki and then every tag as a tree, a nested tag
indented under its parent; choosing one shows its list.

A tag's list is written under `tags/` at the tag's name as written, with a
nested tag's hierarchy as directories: `#[[Notation/Wiki-link]]` is
`tags/Notation/Wiki-link/`. Two spellings share a list only when they are the
same tag, and tags compare exactly, case included: `#[[Rust]]` and `#[[rust]]`
are two tags with two lists. The naming rule is the one in
[[Reference/Static Site/Output Structure]].

The list of every tag is `all-tags/index.html`.
