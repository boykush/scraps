---
type: Reference
description: "`[[key]]` links to the scrap with that key."
---

#[[Notation/Wiki-link]]

`[[key]]` links to the scrap with that key, and is shown as the scrap's
title.

```markdown
See [[Getting Started]] for the basic flow.
```

If the key does not resolve, [[Reference/Lint Rules]] reports a
`broken-link` warning. Tags are a separate namespace and are not implicit
fallback targets.
