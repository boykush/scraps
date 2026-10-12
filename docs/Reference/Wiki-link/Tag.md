#[[Notation/Wiki-link]]

`#[[tag]]` marks a tag. Tags and scraps live in **separate namespaces**: a
scrap named `Markdown` is reached via `[[Markdown]]`, while the tag is
`#[[Markdown]]`.

```markdown
This scrap is about #[[Markdown]] and #[[Notation/Wiki-link]].
```

A scrap's tags are also read from the `tags` list of its YAML frontmatter,
the key the [Open Knowledge Format](https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md)
defines. The two sources are merged, and a tag named in both counts once.

```markdown
---
tags: [Markdown, Notation/Wiki-link]
---
```

The key is optional and read tolerantly: a `tags` value that is not a list,
and entries that are not strings, are ignored without a warning. No other
frontmatter key is read as a tag.

Each tag has a generated index page on the static site — see
[[Reference/Static Site/Tag Pages]].
