#[[Configuration]]

Configuration is managed by `.scraps.toml`. The directory containing this file
is the Scraps wiki root, and every Markdown file under it is treated as a
scrap unless it is in `static/` or the configured output directory.

## Configuration Structure

The configuration file has three areas:

- **Root level**: Contains `output_dir` and `timezone`
- **[ssg] section**: Contains all static site generator settings
- **[lint.*] sections**: Configure opt-in lint rules

The `[ssg]` section is required for `build` and `serve` commands. Other commands
like `lint`, `tag`, and `mcp` can work without this section.

Within the `[ssg]` section, `base_url` and `title` are required fields.

## Configuration Variables

All configuration variables used by Scraps and their default values are listed below.

```toml:.scraps.toml
# Build output directory relative to this .scraps.toml (optional, default=_site)
output_dir = "_site"

# The site timezone (optional, default=UTC)
timezone = "UTC"

# SSG (Static Site Generator) configuration section
# This section is required for build and serve commands
[ssg]
# The site base url (required)
base_url = "https://username.github.io/repository-name/"

# The site title (required)
title = ""

# The site language (compliant with iso639-1, default=en)
lang_code = "en"

# The site description (optional)
description = ""

# The site favicon in the form of png file URL (optional)
favicon = ""

# The site color scheme
# (optional, default=os_setting, choices=os_setting or only_light or only_dark)
color_scheme = "os_setting"

# Build a search index with the Fuse JSON and display search UI
# (optional, default=true, choices=true or false)
build_search_index = true

# Scraps sort key choice on index page
# (optional, default=committed_date, choices=committed_date or linked_count)
sort_key = "committed_date"

# Scraps pagination on index page(optional, default=no pagination)
paginate_by = 20

# Optional lint rule configuration.
# Presence of this section enables stale-by-git during `scraps lint`.
[lint.stale_by_git]
enabled = true
threshold_days = 180
```

## Project Root

Scraps does not use a `scraps_dir` setting in v1. To keep multiple independent
wikis in one repository, place a separate `.scraps.toml` in each wiki directory
and run commands with `-C`:

```bash
❯ scraps -C docs build
❯ scraps -C internal-wiki lint
```

The old `-p` / `--path` flag is still accepted as a deprecated alias for one
release. Prefer `-C` / `--directory` or the `SCRAPS_DIRECTORY` environment
variable.
