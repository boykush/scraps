#[[Notation/Wiki-link]]

`[[Title#Heading]]` links to a specific heading inside another scrap.

```markdown
See [[Configuration#SSG section]] for the schema.
```

If the target scrap exists but the heading does not match,
[[Reference/Lint Rules]] reports `broken-heading-ref`. Heading text
matches by slug, the same form the static site uses for fragment URLs.
