---
type: Reference
---

#[[CLI]]

Link quality rules used by [[Reference/CLI Overview]]'s `lint`
command. Default rules are graph-mechanical (no external dependencies);
rules with dependencies are opt-in via `--rule` or `[lint.<rule>]` in
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
| `path-collision` | scrap whose page shares its output path with another page; `scraps build` fails on it ([[Reference/Static Site/Output Structure]]) | on |
| `stale-by-git` | last commit older than threshold (git-dependent) | opt-in |

Output follows the `cargo clippy`-style diagnostic format. For LLM-driven
purpose-based rule selection, see the `lint-rule-handler` agent in the
[llm-wiki plugin](https://github.com/boykush/scraps/tree/main/plugins/llm-wiki).
