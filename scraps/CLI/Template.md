#[[CLI]] #[[Templates]]

```bash
❯ scraps template
```

This command generates scrap files from Markdown templates located in the `/templates` directory.

## Commands

### List Templates

```bash
❯ scraps template list
```

Lists all available templates in the `/templates` directory.

**Example output:**
```
daily_note
book
meeting
project
```

### Generate Scrap from Template

```bash
❯ scraps template generate <TEMPLATE_NAME> [OPTIONS]
```

Generates a scrap file from the specified template.

**Options:**
- `-t, --title <TITLE>`: Specify the scrap title (required if not specified in template metadata)

**Examples:**

With metadata-specified title:
```bash
❯ scraps template generate daily_note
```

With command-line title:
```bash
❯ scraps template generate meeting -t "Weekly Standup"
```

With environment variables:
```bash
❯ TITLE="My Book Review" scraps template generate book
```

## References

- Template features and syntax: [[Feature/Templates]]
- Template samples: [[Sample templates]]