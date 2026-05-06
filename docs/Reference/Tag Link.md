#[[Wiki-Links]] #[[Markdown]]

Tags use explicit `#[[tag]]` syntax.

```markdown
This scrap is about #[[Markdown]] and #[[Writing/Reference]].
```

Tags and scraps live in separate namespaces. `[[Markdown]]` is a link to a
scrap named `Markdown`; `#[[Markdown]]` is a tag. If `[[Markdown]]` does not
resolve to a scrap, `scraps lint` reports a `broken-link` warning.

Tags are displayed on the index page. Each tag links to a page that lists all scraps using that tag.
