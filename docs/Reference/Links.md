#[[CLI]] #[[Wiki-Links]]

```bash
❯ scraps links <TITLE>
```

This command lists outbound wiki-links from a scrap. Use `--ctx` when the title
needs context disambiguation.

## Examples

```bash
# Table output
❯ scraps links "Getting Started"

# Context-qualified target scrap
❯ scraps links "Service" --ctx Kubernetes

# JSON output
❯ scraps links "Getting Started" --json
```

## JSON Shape

```json
{
  "results": [
    {
      "title": "Configuration",
      "ctx": null
    }
  ],
  "count": 1
}
```
