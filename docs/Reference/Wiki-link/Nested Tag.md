#[[Notation/Wiki-link]]

`#[[a/b/c]]` is a nested tag with **max depth 3**. Nested tags
auto-aggregate Logseq-style: `#[[Notation/Wiki-link]]` and
`#[[Notation/Markdown]]` both surface under the `#[[Notation]]` index.

```markdown
#[[Emit/Static Site]]
#[[Emit/CLI JSON]]
```

A frontmatter `tags` entry is split on `/` the same way, so
`tags: [Emit/Static Site]` is the nested tag `#[[Emit/Static Site]]`.

Each tag (and each parent in the hierarchy) gets its own page on the
static site — see [[Reference/Static Site/Tag Pages]].
