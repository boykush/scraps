#[[CLI]]

```bash
❯ scraps search <QUERY>
```

This command searches scrap titles and Markdown bodies using fuzzy matching.
Space-separated keywords use `or` logic by default.

## Examples

```bash
# Search with default OR logic
❯ scraps search "rust cli"

# Require every keyword to match
❯ scraps search "rust cli" --logic and

# Limit result count
❯ scraps search "rust cli" --num 20

# JSON output for agents and scripts
❯ scraps search "rust cli" --logic and --json
```

## JSON Shape

```json
{
  "results": [
    {
      "title": "Rust CLI",
      "ctx": "Programming"
    }
  ],
  "count": 1
}
```
