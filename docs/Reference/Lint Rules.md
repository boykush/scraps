#[[CLI]]

Wiki-link quality rules used by [[Reference/CLI Overview]]'s `lint`
command. Default rules are graph-mechanical (no external dependencies);
rules with dependencies are opt-in via `--rule` or `[lint.<rule>]` in
[[Reference/Configuration]].

| Rule | Detects | Default |
|---|---|---|
| `dead-end` | scrap with no outbound links | on |
| `lonely` | scrap with no backlinks | on |
| `self-link` | scrap links to itself | on |
| `overlinking` | same `[[link]]` repeated in one scrap | on |
| `broken-link` | `[[link]]` that does not resolve | on |
| `broken-heading-ref` | `[[Page#Heading]]` heading missing | on |
| `stale-by-git` | last commit older than threshold (git-dependent) | opt-in |

Output follows the `cargo clippy`-style diagnostic format. For LLM-driven
purpose-based rule selection, see the `lint-rule-handler` agent in the
[scraps plugin](https://github.com/boykush/scraps/tree/main/plugins/scraps).
