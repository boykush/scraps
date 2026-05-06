#[[Notation/Wiki-link]]

Wiki-link notation gives Markdown a typed surface: each `[[…]]` is a typed
reference that the compiler can resolve, lint, and emit. See
[[Explanation/What is Scraps?#markdown-as-typed-source]] for the design
rationale.

## Normal link

`[[Title]]` resolves by title within the wiki.

```markdown
See [[Getting Started]] for the basic flow.
```

If the title does not resolve, [[Reference/CLI Overview#lint]] reports a
`broken-link` warning. Tags are a separate namespace and are not implicit
fallback targets — use `#[[…]]` (see [Tag](#tag) below) when you mean a tag.

## Alias

`[[Title|Display]]` shows custom display text while linking to `Title`.

```markdown
[[Getting Started|the tutorial]]
```

## Context-qualified link

When two scraps share a title in different folders (see [Context](#context) below), prefix
the link with the context path:

```markdown
[[DDD/Service]]
[[Kubernetes/Service]]
```

Context-qualified links resolve from the wiki root, not relative to the
linking scrap's context. `[[Service]]` always refers to the root `Service.md`.

Combine with alias: `[[Kubernetes/Service|Kubernetes Service]]`.

## Heading reference

`[[Title#Heading]]` links to a specific heading inside another scrap.

```markdown
See [[Configuration#SSG section]] for the full TOML schema.
```

If the target scrap exists but the heading does not match, lint reports
`broken-heading-ref`. Heading text matches against the rendered heading.

## Embed

`![[Title]]` inlines another scrap's body at this location. Embeds remove
duplication when two pages need to show the same content.

```markdown
![[Getting Started]]
```

## Section embed

`![[Title#Heading]]` embeds a single section from another scrap.

```markdown
![[Configuration#SSG section]]
```

This is how this site keeps the [ssg] schema authoritative in
[[Reference/Configuration]] while [[Reference/Static Site]] surfaces the same
block — there is exactly one source of truth.

## Tag

`#[[tag]]` marks a tag. Tags and scraps live in **separate namespaces**: a
scrap named `Markdown` is reached via `[[Markdown]]`, while the tag is
`#[[Markdown]]`.

```markdown
This scrap is about #[[Markdown]] and #[[Notation/Wiki-link]].
```

Each tag has a generated index page on the static site; see
[[Reference/Static Site#tag-pages]].

## Nested tag

`#[[a/b/c]]` is a nested tag with **max depth 3**. Nested tags
auto-aggregate Logseq-style: `#[[Notation/Wiki-link]]` and
`#[[Notation/Markdown]]` both surface under the `Notation` index.

```markdown
#[[Emit/Static Site]]
#[[Emit/CLI JSON]]
```

## Context

Folder structure under the wiki root becomes a scrap's context:

```bash
.
├── DDD/
│   └── Service.md      # ctx = ["DDD"], title = "Service"
└── Kubernetes/
    └── Service.md      # ctx = ["Kubernetes"], title = "Service"
```

Context depth is bounded at **3** segments. Context is part of the scrap's
identity, not a disambiguator on top of a flat name-keyed identity — two
scraps with identical title + ctx would be a hard error.

Use folders only when titles collide or when a scrap is strongly bound to a
context. Avoid deep nesting for browsing convenience alone.

## Resolution rules

- **Title match.** `[[name]]` resolves by exact title against scraps in the
  wiki.
- **Ambiguity is an error.** If `[[Service]]` could match
  `DDD/Service.md` or `Kubernetes/Service.md`, the build / lint fails with a
  resolution error. Disambiguate with `[[DDD/Service]]`.
- **Tags do not fall back.** A `[[name]]` that does not match a scrap is
  always `broken-link`, even if a `#[[name]]` tag exists.
- **Heading match.** `[[Title#Heading]]` matches against the target scrap's
  rendered headings.

For the lint rules that enforce these, see [[Reference/CLI Overview#lint]].
