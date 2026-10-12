---
type: Reference
---

#[[Notation/Frontmatter]]

Scraps reads a YAML frontmatter block when a file has one, and requires
none. Two keys carry meaning; every other key is kept as data and given no
meaning.

```markdown
---
type: Blog
tags: [Markdown, Notation/Wiki-link]
owner: alice
---

The body starts here.
```

## Keys Scraps reads

| Key | Read as |
|---|---|
| `tags` | The scrap's tags, merged with the `#[[tag]]` references in its body |
| `type` | The scrap's kind |

### `tags`

A list of strings. Each one is a tag exactly as `#[[tag]]` is, including
`/` for nesting, and a tag named in both places counts once. See
[[Reference/Wiki-link/Tag]] and [[Reference/Wiki-link/Nested Tag]].

### `type`

A single string saying what the scrap *is*, where a tag says what it is
*about*: a scrap can carry many tags and at most one type. `type: Blog` on
every blog summary replaces the habit of linking each of them to a `Blog`
hub scrap only to group them.

Scraps keeps no list of valid types and attaches no behaviour to any value,
so `Blog` and `blog` are two types. A scrap without a type is simply untyped.

`scraps type list` shows the types in use and `scraps type scraps <type>`
the scraps of one; `scraps get --json type` returns a single scrap's type.
This documentation types its own pages this way: run `scraps type list` in
`docs/`.

On the static site a type narrows the list views; see
[[Reference/Static Site/Type Filter]].

## Reading rules

Frontmatter never fails a command and never raises a lint warning.

- A block that is absent, unclosed or not valid YAML is skipped; the scrap is
  read as if it had none, and the lines stay ordinary Markdown.
- A key whose value has the wrong shape is ignored: `tags` that is not a
  list, a `tags` entry that is not a string, a `type` that is not a non-empty
  string.
- Keys Scraps does not read are passed through untouched, in the order they
  were written.

## What frontmatter does not change

A scrap's title and context come from its file path, and its links from
[[Reference/Wiki-link Notation]]. No frontmatter key overrides them.

## Where the block appears

A block Scraps reads is metadata, not content:

- The static site leaves it out of the scrap's page, of the summary on index
  and tag pages, and of any page that embeds the scrap.
- Nothing in it is read as structure: no heading, code block, image, task
  or `[[ ]]` reference comes out of the YAML, and `scraps search` does not
  match its text.
- Line numbers reported for the rest of the file still count the block's
  lines, so they are the lines of the file.

The `body` of `scraps get --json` and of the MCP `get_scrap` tool is the
exception: it is the file's text, block included.

## Open Knowledge Format

`tags` and `type` are the keys the
[Open Knowledge Format](https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md)
(OKF) defines, so files written for OKF bring their tags and types with them.
The reverse is not asked for: OKF requires `type` on every document, Scraps
on none.

The rest of OKF is not interpreted. Its other keys are passed through like
any author-defined key, links between files written as ordinary Markdown
links are not Scraps links, and `index.md` and `log.md` are ordinary scraps.

## Reading the whole block

`scraps frontmatter --json` returns every scrap that carries a block with
all of its keys, read or not — see [[Reference/CLI Overview]].
