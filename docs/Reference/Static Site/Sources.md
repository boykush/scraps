---
type: Reference
---

#[[Emit/Static Site]]

A scrap page whose frontmatter has readable `sources` lists them after the
body, in the order written, under a `sources` heading with their count. A
scrap without any shows no list. The key itself is described in
[[Reference/Frontmatter]].

Each entry is labelled by its `title`, or by its `resource` when it has
none. An `http` or `https` URL becomes a link to that address. Any other
`resource` — a path, a scrap's key, a description of something no one can
follow — is shown as text, after the `title` when there is one. A path
naming a scrap is still a link to it: the page lists that scrap among its
links, and the scrap's page lists this one among its backlinks. `id`,
`author` and `last_modified` are not shown.
