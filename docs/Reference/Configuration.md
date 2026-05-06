#[[Configuration]]

`.scraps.toml` declares a Scraps wiki: the directory containing this file is
the wiki root, and every Markdown file under it is a scrap unless it lives in
`static/` or the configured `output_dir`.

See [[Explanation/What is Scraps?]] for why config discovery follows this shape,
and [[Reference/CLI Overview]] for how `-C` selects between multiple
`.scraps.toml` directories.

## Areas

`.scraps.toml` has three areas:

| Area | Purpose | Required for |
|---|---|---|
| Root level | wiki-wide settings | every command |
| `[ssg]` | static-site emit target | `build`, `serve` |
| `[lint.*]` | opt-in lint rule config | `lint` (only those rules) |

The `[ssg]` section is required only for `build` and `serve`; `lint`, `tag`,
`get`, `search`, and `mcp serve` work without it. Within `[ssg]`, `base_url`
and `title` are required.

## Root level

```toml:.scraps.toml
# Build output directory relative to this .scraps.toml (optional, default=_site)
output_dir = "_site"

# The site timezone (optional, default=UTC)
timezone = "UTC"
```

## SSG section

Used by [[Reference/Static Site]] (HTML emit target).

```toml:.scraps.toml
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

# Scraps pagination on index page (optional, default=no pagination)
paginate_by = 20
```

## Lint rules

Opt-in rules read their config from `[lint.<rule>]`. Presence of the section
enables the rule for `scraps lint` without requiring `--rule`.

```toml:.scraps.toml
# Enables stale-by-git during `scraps lint`.
[lint.stale_by_git]
enabled = true
threshold_days = 180
```

Default (graph-mechanical) rules are always on; see [[Reference/CLI Overview#lint]].

## Project Root

Scraps does not use a `scraps_dir` setting in v1. To keep multiple independent
wikis in one repository, place a separate `.scraps.toml` in each wiki directory
and run commands with `-C`:

```bash
❯ scraps -C docs build
❯ scraps -C internal-wiki lint
```

Each `.scraps.toml` is its own independent wiki — they do not cross-link, and
each builds to its own `output_dir`.

The old `-p` / `--path` flag is still accepted as a deprecated alias for one
release. Prefer `-C` / `--directory` or the `SCRAPS_DIRECTORY` environment
variable.
