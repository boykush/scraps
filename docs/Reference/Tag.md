#[[CLI]]

```bash
❯ scraps tag list
```

This command lists all tags found in your Scraps content, helping you understand the tag distribution across your knowledge base.

Use `scraps tag backlinks <TAG>` to list scraps that reference a specific tag.

## Examples

```bash
# List all tags
❯ scraps tag list

# JSON output
❯ scraps tag list --json

# List scraps tagged with rust
❯ scraps tag backlinks rust

# List tags from specific directory
❯ scraps -C /path/to/wiki tag list
```
