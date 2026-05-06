#[[CLI]]

```bash
❯ scraps get <TITLE>
```

This command prints the Markdown body of one scrap. Use `--ctx` when multiple
scraps share the same title in different context folders.

## Examples

```bash
# Print Markdown
❯ scraps get "Getting Started"

# Disambiguate by context
❯ scraps get "Service" --ctx Kubernetes

# JSON output for agents and scripts
❯ scraps get "Getting Started" --json
```

## JSON Shape

```json
{
  "title": "Getting Started",
  "ctx": null,
  "md_text": "# Getting Started\n...",
  "headings": [
    {
      "level": 1,
      "text": "Getting Started",
      "line": 1
    }
  ],
  "code_blocks": [
    {
      "lang": "bash",
      "content": "scraps build",
      "line": 12
    }
  ]
}
```
