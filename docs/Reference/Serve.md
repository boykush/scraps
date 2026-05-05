#[[CLI]]

```bash
❯ scraps serve
```

This command starts a local development server to preview your static site. The server automatically serves the files from the build output directory (`_site` by default, configurable via `output_dir` in `.scraps.toml`) at [http://127.0.0.1:1112](http://127.0.0.1:1112).

## Examples

```bash
# Basic serve
❯ scraps serve

# Serve from specific directory
❯ scraps serve --path /path/to/project
```

Use this command to check how your site looks and functions before deployment.
