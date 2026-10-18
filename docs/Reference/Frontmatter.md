---
type: Reference
---

#[[Notation/Frontmatter]]

Scraps reads a YAML frontmatter block when a file has one, and requires
none. Six keys carry meaning; every other key is kept as data and given no
meaning.

```markdown
---
title: Why wiki-links?
type: Blog
description: Why wiki-links beat Markdown links for notes.
tags: [Markdown, Notation/Wiki-link]
owner: alice
---

The body starts here.
```

## Keys Scraps reads

| Key | Read as |
|---|---|
| `title` | The scrap's title: the name readers see |
| `tags` | The scrap's tags, merged with the `#[[tag]]` references in its body |
| `type` | The scrap's kind |
| `description` | The scrap's one-line summary |
| `verified` | Who confirmed the scrap's content, and when |
| `sources` | The materials the scrap's content derives from |

### `title`

A single string: the name readers see. Surrounding whitespace is trimmed and
the string is otherwise used as written, so it can hold spaces and symbols a
file name is better off without. A scrap without one is titled by its file
name without `.md`.

The static site shows the title wherever it names the scrap: the page
heading and `<title>`, the list views, tag pages, link lists, the
neighbourhood graph, the search box, and the text of a `[[key]]` written
without an alias. The `title` field of every `--json` output is this title,
and `scraps search` matches it.

A title names a scrap and does not identify it: links, commands and URLs go
by its key. The model is in
[[Reference/Wiki-link Notation#Key, title and context]].

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

### `description`

A single string: one sentence summarising the scrap, written for a reader
deciding whether to open it. Surrounding whitespace is trimmed and the
sentence is otherwise used as written.

On the static site it replaces the summary derived from the body on the
updated and backlinks views, and on the scrap's own page it fills the
`description`, `og:description` and `twitter:description` meta tags in place
of the site-wide `description` of [[Reference/Configuration]]. A scrap
without one keeps the derived summary and the site-wide value.

`scraps get --json description` returns a single scrap's description, and
each `scraps search --json` hit that has one carries it.

### `verified`

A list of confirmations, each a mapping with `by`, who or what confirmed the
content, and `at`, when. A single confirmation may be written as one bare
mapping, read as a list of one.

```yaml
verified:
  - by: human:alice
    at: 2026-10-17T09:00:00+09:00
  - by: process:ci
    at: 2026-10-16T00:00:00Z
```

`by` is an actor as the Open Knowledge Format writes one: `human:<id>` for a
person, `process:<id>` for an automated process, `<producer>/<version>` for an
agent. `at` is an RFC 3339 datetime with an explicit offset.

From the readable entries Scraps derives the scrap's trust tier, lowest
first:

| Tier | When |
|---|---|
| `unverified` | No readable entry |
| `machine-confirmed` | Entries, none by a `human:` actor |
| `human-reviewed` | At least one entry by a `human:` actor |

`scraps get --json verified` returns `{ tier, entries }`, with `entries` the
readable confirmations newest first and each `at` in UTC. A scrap without
the key is `unverified` with no entries. The tier is advisory: nothing in
Scraps is withheld or reordered by it.

### `sources`

A list of mappings, one per material the content derives from. Each entry
needs `resource`; the other keys are optional. A single source may be
written as one bare mapping, read as a list of one.

```yaml
sources:
  - resource: https://example.com/handbook
    title: Team handbook
    author: human:alice
    last_modified: 2026-05-30T00:00:00Z
  - resource: references/ga4-schema.md
    id: ga4-schema
```

| Key | Read as |
|---|---|
| `resource` | What the content derives from, as written |
| `id` | A stable key the body can cite the source by |
| `title` | A label for readers |
| `author` | Who wrote the source, as an actor like `by` of `verified` |
| `last_modified` | When the source last changed, an RFC 3339 datetime with an explicit offset |

`resource` may be a URL, a path, or a description of something no one can
follow, and OKF marks none of them, so Scraps keeps it as written. When it is
a path to a `.md` file, it is also a link to that scrap, resolved as a
Markdown link is: from the wiki root when it starts with `/`,
otherwise from the folder holding the scrap.

```yaml
sources:
  - resource: /Blog/Introducing the Open Knowledge Format.md
```

Such a link counts as any other: it is among the scrap's links and the
target's backlinks, and a path naming no scrap is a `broken-link`. What tells
it apart is its kind, `source`, which `scraps links` and `scraps backlinks`
report next to `link` and `embed`.

`scraps get --json sources` returns the readable entries in the order
written, each with only the keys it carries and `last_modified` in UTC. A
scrap without the key returns an empty list. Sources do not change the
trust tier of `verified`. OKF's `usage_count` and `usage_window` are not
read.

The static site lists the sources after the scrap's body, linking those
whose `resource` is a web URL; see [[Reference/Static Site/Sources]].

## Reading rules

Frontmatter never fails a command and raises no lint warning of its own:
the one warning it can lead to is a `broken-link` for a `sources` path
naming no scrap.

- A block that is absent, unclosed or not valid YAML is skipped; the scrap is
  read as if it had none, and the lines stay ordinary Markdown.
- A key whose value has the wrong shape is ignored: `tags` that is not a
  list, a `tags` entry that is not a string, a `title`, `type` or
  `description` that is not a non-empty string, a `verified` entry without
  a non-empty `by` or whose `at` is not a datetime with an offset, a
  `sources` entry without a non-empty `resource`. Within a readable
  `sources` entry, an optional key of the wrong shape is dropped and the
  entry kept.
- Keys Scraps does not read are passed through untouched, in the order they
  were written.

## What frontmatter does not change

A scrap's key and context come from its file path, and so do the links that
resolve to it and the URL of its page. Its own links come from the `[[ ]]`
references and the Markdown links to scrap files in its body. No frontmatter
key overrides any of these: `title` changes what a scrap is called, never
what points at it.

## Where the block appears

A block Scraps reads is metadata, not content:

- The static site leaves it out of the scrap's page, of the summary derived
  for index pages, and of any page that embeds the scrap. Only a `title`, a
  `description` and `sources` are shown, where the keys say so above.
- Nothing in it is read as structure: no heading, code block, image, task,
  `[[ ]]` reference or Markdown link comes out of the YAML, and
  `scraps search` does not match its text, the `title` apart.
- The `body` of `scraps get`, with or without `--json`, and of the MCP
  `get_scrap` tool is the content without the block: it starts at the first
  content line, and the blank lines after the block go with it.
- Line numbers reported for the rest of the file still count the block's
  lines, so they are the lines of the file. They are not offsets into
  `body`, which is shorter by the block.

`scraps get --json frontmatter`, or `fields: ["frontmatter"]` on `get_scrap`,
returns the block of one scrap as data: its keys in the order they were
written, or `null` when the scrap has none.

## Open Knowledge Format

`title`, `tags`, `type`, `description`, `verified` and `sources` are keys the
[Open Knowledge Format](https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md)
(OKF) defines, so files written for OKF bring their titles, tags, types,
descriptions, confirmations and sources with them. Scraps reads `verified` as an OKF
consumer does, deriving the trust tier from it alone. OKF's concept ID, the file path without `.md`, is
the scrap's key.
The reverse is not asked for: OKF requires `type` on every document, Scraps
on none.

OKF connects files with ordinary Markdown links to their `.md` paths, and
those are links between scraps: see [[Reference/Markdown Link]].

The rest of OKF is not interpreted. Its other keys are passed through like
any author-defined key, and `index.md` and `log.md` are ordinary scraps.

## Reading the whole block

`scraps frontmatter --json` returns every scrap that carries a block with
all of its keys, read or not, and `scraps get --json frontmatter` the block
of a single scrap — see [[Reference/CLI Overview]].
