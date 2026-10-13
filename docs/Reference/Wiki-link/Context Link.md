---
type: Reference
---

#[[Notation/Wiki-link]]

`[[ctx/name]]` is the key of a scrap in a folder: the folder path, then the
file name without `.md`. It tells apart scraps whose file names are the same
in different folders.

```markdown
[[DDD/Service]]
[[Kubernetes/Service]]
```

A key is written from the wiki root, not relative to the linking scrap. Context depth is bounded at 3 segments. Combine with
[[Reference/Wiki-link/Alias]] when you want a different display text.
