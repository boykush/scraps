#[[CLI]]

```bash
❯ scraps search <QUERY>
```

This command searches through your Scraps content using fuzzy matching to find relevant information across your knowledge base.

## Examples

```bash
# Basic search
❯ scraps search "markdown"

# Limit results to 10
❯ scraps search "documentation" --num 10
```

The search uses fuzzy matching across file names, content, and Wiki-link references, displaying results ranked by relevance.