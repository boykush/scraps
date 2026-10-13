---
type: Reference
---

#[[Notation/Wiki-link]]

Wiki-link notation gives Markdown a typed surface: each `[[…]]` is a typed
reference that the compiler can resolve, lint, and emit. See
[[Explanation/What is Scraps?#markdown-as-typed-source]] for the design
rationale.

## [[Reference/Wiki-link/Normal Link|Normal link]]

![[Reference/Wiki-link/Normal Link]]

## [[Reference/Wiki-link/Alias|Alias]]

![[Reference/Wiki-link/Alias]]

## [[Reference/Wiki-link/Context Link|Context-qualified link]]

![[Reference/Wiki-link/Context Link]]

## [[Reference/Wiki-link/Heading Reference|Heading reference]]

![[Reference/Wiki-link/Heading Reference]]

## [[Reference/Wiki-link/Embed|Embed]]

![[Reference/Wiki-link/Embed]]

## [[Reference/Wiki-link/Section Embed|Section embed]]

![[Reference/Wiki-link/Section Embed]]

## [[Reference/Wiki-link/Tag|Tag]]

![[Reference/Wiki-link/Tag]]

## [[Reference/Wiki-link/Nested Tag|Nested tag]]

![[Reference/Wiki-link/Nested Tag]]

## Key, title and context

A scrap has one identity and one name.

| | What it is | Where it comes from |
|---|---|---|
| Key | What a `[[ ]]` holds, and the handle commands and tools take and return | The file's path under the wiki root without `.md`: `Getting Started`, `tables/orders` |
| Title | What readers see | The frontmatter `title` when the scrap has one, otherwise the file name without `.md` |
| Context | Where the scrap lives: `tables` in `tables/orders` | The folder part of the key |

The filesystem keeps keys unique, so two scraps can share a title and never
a key. Links resolve by key alone: with `title: Customer Orders` in
`tables/orders.md`, the scrap is linked as `[[tables/orders]]` and shown as
"Customer Orders", and `[[Customer Orders]]` is a `broken-link`.

A scrap needs no frontmatter to be named: `Getting Started.md` has the key
and the title `Getting Started`. A tidy file name with a `title` is the way
to get a clean URL and a free-text name at once; the key decides the page's
path ([[Reference/Static Site/Output Structure]]) and the title decides what
the page is called ([[Reference/Frontmatter]]).

## Resolution rules

- **Key match.** `[[name]]` resolves by exact key: `[[Getting Started]]` is
  `Getting Started.md` at the wiki root and `[[tables/orders]]` is
  `tables/orders.md`. A title is never matched.
- **No search across folders.** `[[Service]]` names `Service.md` at the wiki
  root only. With `DDD/Service.md` and `Kubernetes/Service.md` and no root
  file, it is a `broken-link`.
- **Tags do not fall back.** A `[[name]]` that does not match a scrap is
  always `broken-link`, even if a `#[[name]]` tag exists.
- **Heading match.** `[[key#Heading]]` matches by slug against the
  target's rendered headings.
- **`#` and `|` are syntax.** A scrap whose file or folder name contains
  either cannot be the target of a `[[…]]`; [[Reference/Lint Rules]] reports
  it as `unlinkable-key`.
