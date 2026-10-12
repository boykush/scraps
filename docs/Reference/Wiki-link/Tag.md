#[[Notation/Wiki-link]]

`#[[tag]]` marks a tag. Tags and scraps live in **separate namespaces**: a
scrap named `Markdown` is reached via `[[Markdown]]`, while the tag is
`#[[Markdown]]`.

```markdown
This scrap is about #[[Markdown]] and #[[Notation/Wiki-link]].
```

A scrap's tags are also read from the `tags` list of its YAML frontmatter
and merged with the ones in its body — see [[Reference/Frontmatter]].

Each tag has a generated index page on the static site — see
[[Reference/Static Site/Tag Pages]].
