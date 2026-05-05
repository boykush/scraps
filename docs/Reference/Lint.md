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

### singleton-tag

Detects tags referenced by only 1 scrap. Tags used by a single scrap provide no grouping value and may indicate a tag that should be removed or consolidated.

## Output Format

Diagnostics follow the same style as `cargo clippy`:

```
warning[dead-end]: scrap has no links to other scraps
 --> orphan.md

warning[overlinking]: link [[Rust]] appears 3 times
 --> programming.md:2:5
  |
2 | See [[Rust]] for details. Also [[Rust]] and [[Rust]].
  |     ^^^^^^^^
  |
```

## Examples

```bash
# Lint current project
❯ scraps lint

# Lint from specific directory
❯ scraps lint --path /path/to/project
```
