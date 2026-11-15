---
type: Reference
---

#[[CLI]]

Link quality rules used by [[Reference/CLI Overview]]'s `lint`
command. Default rules report what is a defect in any wiki: a link that does
not resolve, a scrap that cannot be linked. Opt-in rules check a policy the
wiki chooses for itself, such as how old is stale or which filesystems it
must port to; writing the rule's `[lint.<rule>]` section in
[[Reference/Configuration]] turns it on, and `--rule` runs it once. A link is a `[[wiki-link]]` or a Markdown link
to a scrap's `.md` file ([[Reference/Markdown Link]]); the rules treat both
alike and quote each as it was written.

| Rule | Detects | Default |
|---|---|---|
| `dead-end` | scrap with no outbound links | on |
| `lonely` | scrap with no backlinks | on |
| `self-link` | scrap links to itself | on |
| `overlinking` | same scrap linked more than once in one scrap | on |
| `broken-link` | link, or `sources` path, that does not resolve to a scrap | on |
| `broken-heading-ref` | `[[Page#Heading]]` or `page.md#heading` heading missing | on |
| `unlinkable-key` | scrap whose file path cannot be written inside `[[ ]]` | on |
| `source-has-page` | `sources` URL that another scrap's `resource` names | on |
| `stale-by-git` | policy: last commit older than the threshold the wiki sets | opt-in |
| `unportable-key` | policy: file path that does not port across filesystems | opt-in |
| `unlinked-mention` | policy: prose that writes another scrap's title without linking it | opt-in |

## Sources

`source-has-page` reports a `sources` entry whose `resource` is the same
string as another scrap's top-level `resource`: the material has a page of
its own, so the entry can point at it by path, `/<key>.md`, and become a
`source` link. A scrap naming its own `resource` in its `sources` is not
reported. See [[Reference/Frontmatter]].

## Mentions

`unlinked-mention` reports where a scrap's prose writes another scrap's
title without linking to it, once per scrap and title, at the first place:

```
warning: unlinked-mention: mentions "Customer Orders" without linking [[tables/orders]]
```

Whether a word that names a scrap should link to it is the wiki's call, so
the rule is opt-in:

```toml:.scraps.toml
[lint.unlinked_mention]
```

What it counts as a mention:

- Only running prose. Links of every kind and their text, images, code,
  raw HTML, bare URLs and the frontmatter block are left out.
- A title is matched as written, case included. One that starts or ends in a
  Latin letter or digit must stand as a whole word, so `Git` is not found in
  `GitHub`. A title in a script written without spaces between words is
  found wherever it is written, so it can be part of a longer word.
- Where titles overlap, the longest wins: `Claude Code` is reported, not
  `Claude`.
- A scrap that already links the target anywhere, and the target itself, are
  not reported.
- Titles shorter than three characters, and a title two scraps share, are
  never matched.

The warning names the target by its key, so the warnings about one scrap are
those that end in its `[[key]]`.

## Key rules

The `*-key` rules look at a scrap's file path under the wiki root, not at its
links, and report at the file. Scraps places no restriction on the characters
in a path; these rules say what a given path costs.

`unlinkable-key` reports a scrap that `[[ ]]` cannot reach, because writing
its path there reads back as something else: `#` starts a heading reference
and `|` an alias, so `[[C#]]` links to `C`, and surrounding whitespace is
trimmed. Such a scrap still builds and can be linked with a Markdown link to
its file, percent-encoded where needed (`[C#](C%23.md)`). Renaming the file
removes the warning, and a frontmatter `title` keeps the name readers see:
`c-sharp.md` with `title: "C#"`.

`unportable-key` reports a scrap whose path cannot exist on every common
filesystem. Whether a wiki must port is its own policy, so the rule is
opt-in:

```toml:.scraps.toml
[lint.unportable_key]
```

| Checked on | Reports | Because |
|---|---|---|
| each folder and file name | `< > : " \ \| ? *` or a control character | Windows does not allow them in a name |
| each folder and file name | a trailing space or dot | Windows drops them from a name |
| each folder and file name | a device name such as `CON`, `NUL` or `COM1`, with or without an extension | Windows reserves them |
| each folder and file name | more than 255 bytes in UTF-8 | most filesystems limit a name to that |
| the whole path | paths that differ only in case, each reported once naming the others | they cannot coexist on the case-insensitive volumes macOS and Windows use by default |

Names are the path's segments without the `.md` extension. Case is compared
by Unicode lowercasing.

Output follows the `cargo clippy`-style diagnostic format. For LLM-driven
purpose-based rule selection, see the `lint-rule-handler` agent in the
[llm-wiki plugin](https://github.com/boykush/scraps/tree/main/plugins/llm-wiki).
