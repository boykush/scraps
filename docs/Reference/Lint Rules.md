---
type: Reference
---

#[[CLI]]

Link quality rules used by [[Reference/CLI Overview]]'s `lint`
command. Default rules need nothing outside the wiki and apply to any wiki;
the others are opt-in via `--rule` or `[lint.<rule>]` in
[[Reference/Configuration]]. A link is a `[[wiki-link]]` or a Markdown link
to a scrap's `.md` file ([[Reference/Markdown Link]]); the rules treat both
alike and quote each as it was written.

| Rule | Detects | Default |
|---|---|---|
| `dead-end` | scrap with no outbound links | on |
| `lonely` | scrap with no backlinks | on |
| `self-link` | scrap links to itself | on |
| `overlinking` | same scrap linked more than once in one scrap | on |
| `broken-link` | link that does not resolve to a scrap | on |
| `broken-heading-ref` | `[[Page#Heading]]` or `page.md#heading` heading missing | on |
| `unlinkable-key` | scrap whose file path cannot be written inside `[[ ]]` | on |
| `stale-by-git` | last commit older than threshold (git-dependent) | opt-in |
| `unportable-key` | scrap whose file path does not port across filesystems | opt-in |

## Key rules

The `*-key` rules look at a scrap's file path under the wiki root, not at its
links, and report at the file. Scraps places no restriction on the characters
in a path; these rules say what a given path costs.

`unlinkable-key` reports a scrap that `[[ ]]` cannot reach, because writing
its path there reads back as something else: `#` starts a heading reference
and `|` an alias, so `[[C#]]` links to `C`, and surrounding whitespace is
trimmed. Such a scrap still builds and can be linked with a Markdown link to
its file, percent-encoded where needed (`[C#](C%23.md)`); renaming the file
removes the warning.

`unportable-key` reports a scrap whose path cannot exist on every common
filesystem. It is opt-in, for a wiki that is cloned onto more than one
system:

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
