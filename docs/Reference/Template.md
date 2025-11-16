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

## Examples

```bash
# List available templates
❯ scraps template list

# Generate from template with metadata-specified title
❯ scraps template generate daily_note

# Generate with command-line title
❯ scraps template generate meeting -t "Weekly Standup"

# Generate with environment variables
❯ TITLE="My Book Review" scraps template generate book
```

## References

- Template features and syntax: [[Explanation/Template System]]
- Template samples: [[How-to/Use Templates]]
