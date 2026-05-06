#[[CLI]] #[[Wiki-Links]]

```bash
❯ scraps backlinks <TITLE>
```

This command lists inbound wiki-links to a scrap. Use `--ctx` when the target
title needs context disambiguation.

## Examples

```bash
# Table output
❯ scraps backlinks "Configuration"

# Context-qualified target scrap
❯ scraps backlinks "Service" --ctx Kubernetes

# JSON output
❯ scraps backlinks "Configuration" --json
```

## JSON Shape

```json
{
  "results": [
    {
      "title": "Getting Started",
      "ctx": null
    }
  ],
  "count": 1
}
```
