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

## Resolution rules

- **Title match.** `[[name]]` resolves by exact title against scraps in the
  wiki.
- **Ambiguity is an error.** If `[[Service]]` could match `DDD/Service.md`
  or `Kubernetes/Service.md`, the build / lint fails with a resolution
  error.
- **Tags do not fall back.** A `[[name]]` that does not match a scrap is
  always `broken-link`, even if a `#[[name]]` tag exists.
- **Heading match.** `[[Title#Heading]]` matches by slug against the
  target's rendered headings.
