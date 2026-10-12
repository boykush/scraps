---
type: Reference
---

#[[Notation/Markdown]]

A Markdown link whose destination is the path of another scrap's `.md` file
is a link between scraps, the same as a `[[wiki-link]]`. Files that connect
documents this way, as the
[Open Knowledge Format](https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md)
does, are read as they are; nothing has to be rewritten into `[[ ]]`.

```markdown
Joined with [customers](/tables/customers.md) and
[revenue](../metrics/revenue.md#definition).
```

## Forms accepted

An inline link `[text](path.md)` or a reference link `[text][label]` is a
scrap link when its destination has no URL scheme and its path ends in
`.md`.

| Destination | Resolved from |
|---|---|
| `/tables/customers.md` | the wiki root, the directory holding `.scraps.toml` |
| `./other.md`, `other.md` | the directory of the linking file |
| `../metrics/revenue.md` | the parent of that directory |

## Resolution

- `.` and `..` segments are normalised. A path that climbs out of the wiki
  root names no scrap.
- Percent-encoding is decoded, so `Static%20Site/Pipeline.md` reaches the
  scrap in the `Static Site` folder. A destination in angle brackets,
  `<Static Site/Pipeline.md>`, does the same without encoding.
- The path names a scrap the way a file does: its folders are the context
  and its file name without `.md` is the title.
- A `#fragment` is a heading reference, as in `[[Title#Heading]]`.
- The link text plays the role of the alias in `[[Title|alias]]`.

## What a scrap link does

Everything a wiki-link does. It is listed by `scraps links` and
`scraps backlinks` and by the MCP link tools, it is an edge of the
neighbourhood graph and a backlink on the static site, and every link rule
of [[Reference/Lint Rules]] applies to it. A link to a `.md` path with no
scrap behind it is a `broken-link`.

On the static site the `href` of a resolved link is rewritten to the page of
the target scrap, the URL a wiki-link to it gets, heading included. A link
that resolves to nothing is left as written.

## What is not a scrap link

These stay ordinary Markdown and are rendered untouched:

- a destination with a scheme, such as `https://example.com/page.md` or
  `mailto:`, or one that starts with `//`
- a destination whose path does not end in `.md`, a query string included
- an image, `![alt](figure.md)`
- anything inside a code span or a fenced code block
- anything inside a frontmatter block ([[Reference/Frontmatter]])
