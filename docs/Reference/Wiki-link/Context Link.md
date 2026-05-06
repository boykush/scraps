#[[Notation/Wiki-link]]

`[[Ctx/Title]]` resolves a scrap whose title is shared with another in a
different folder.

```markdown
[[DDD/Service]]
[[Kubernetes/Service]]
```

Context-qualified links resolve from the wiki root, not relative to the
linking scrap. Context depth is bounded at 3 segments. Combine with
[[Reference/Wiki-link/Alias]] when you want a different display text.
