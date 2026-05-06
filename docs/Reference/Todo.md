#[[CLI]] #[[Markdown]]

```bash
❯ scraps todo
```

This command aggregates GitHub-flavored Markdown task list items across the
wiki. Open tasks are shown by default.

## Status Filters

- `open`: `- [ ] task`
- `done`: `- [x] task`
- `deferred`: `- [-] task`
- `all`: all task items

## Examples

```bash
# Open tasks
❯ scraps todo

# Completed tasks
❯ scraps todo --status done

# JSON output for agents and scripts
❯ scraps todo --status all --json
```

## JSON Shape

```json
{
  "results": [
    {
      "scrap": {
        "title": "Release",
        "ctx": "Planning"
      },
      "status": "open",
      "text": "write release notes",
      "line": 3
    }
  ],
  "count": 1
}
```
