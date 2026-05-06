#[[CLI]]

```bash
❯ scraps lint
```

This command checks your scraps for wiki-link quality issues. It focuses on graph-level problems specific to wiki-link notation, complementing general-purpose Markdown linters like markdownlint.

## Rules

### dead-end

Detects scraps with no outgoing links to other scraps. Dead-end pages break the interconnected nature of a knowledge base and may indicate incomplete content.

### lonely

Detects scraps that are not linked from any other scrap. Lonely pages are difficult to discover through navigation and may indicate content that needs to be integrated into the knowledge graph.

### self-link

Detects scraps that contain a `[[link]]` to themselves. Self-links are redundant as the reader is already on the page.

### overlinking

Detects the same `[[link]]` appearing multiple times within a single scrap. Repeated links add visual clutter without aiding navigation, as readers only need one link to reach the target.

`[[Page|alias]]` and `[[Page]]` are treated as the same link.

### broken-link

Detects `[[link]]` references that do not resolve to an existing scrap. Tags are
not implicit fallback targets in v1; write `#[[tag]]` when you mean a tag.

### broken-heading-ref

Detects `[[Page#Heading]]` references where the scrap exists but the heading
fragment does not match any heading in that target scrap.

### stale-by-git

Detects scraps whose latest git commit is older than a threshold. This rule is
opt-in because it depends on git metadata. Enable it with `--rule stale-by-git`
or with `[lint.stale_by_git]` in `.scraps.toml`.

## Output Format

Diagnostics follow the same style as `cargo clippy`:

```
warning[dead-end]: scrap has no links to other scraps
 --> /path/to/wiki/orphan.md

warning[overlinking]: link [[Rust]] appears 3 times
 --> /path/to/wiki/programming.md:2:5
  |
2 | See [[Rust]] for details. Also [[Rust]] and [[Rust]].
  |     ^^^^^^^^
  |
```

## Examples

```bash
# Lint current project
❯ scraps lint

# Run one rule
❯ scraps lint --rule broken-link

# Run opt-in stale check
❯ scraps lint --rule stale-by-git

# Lint from specific directory
❯ scraps -C /path/to/wiki lint
```
