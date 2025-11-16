#[[Templates]]

Generate scrap files from predefined Markdown templates for efficient content creation.

## Basic Usage

1. Create template files in `/templates` directory
2. Run generate scrap on [[Reference/Template|command-line]]

## Template Syntax

Templates use [Tera](https://keats.github.io/tera) template engine with TOML metadata:

```markdown
+++
title = "{{ now() | date(timezone=timezone) }}"
+++

# Content goes here
```

## Available Variables

- `timezone` - Access Config.toml timezone setting
- All [Tera built-in functions](https://keats.github.io/tera/docs/#built-in-functions)

## Examples

See [[How-to/Use Templates]] for ready-to-use templates.

For CLI commands, see [[Reference/Template]].
