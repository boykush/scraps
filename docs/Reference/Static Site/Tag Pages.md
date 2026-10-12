---
type: Reference
---

#[[Emit/Static Site]]

Each tag carried by at least one scrap, as `#[[tag]]` in its body or in its
frontmatter `tags` list, gets a generated index page listing all scraps that
carry it. Nested tags (`#[[a/b/c]]`)
auto-aggregate to their parents — see [[Reference/Wiki-link/Nested Tag]].

A tag's page is written under `tags/` at the tag's name as written, with a
nested tag's hierarchy as directories: `#[[Notation/Wiki-link]]` is
`tags/Notation/Wiki-link.html`. Two spellings share a page only when they are
the same tag, and tags compare exactly, case included: `#[[Rust]]` and
`#[[rust]]` are two tags with two pages. The naming rule is the one in
[[Reference/Static Site/Output Structure]].

The list of every tag is `all-tags/index.html`.
