#[[Configuration]]

Configuration is managed by `Config.toml` in the Scraps project.

See [[Tutorial/Configure Project]] for a quick setup guide.

## Configuration Variables

The `base_url` and `title` variables are required when using Scraps as a static
site generator (SSG). Other commands like `tag`, `mcp`, and `template` can work
without these fields. All configuration variables used by Scraps and their
default values are listed below.

```toml:Config.toml
# The site base url (required for SSG)
base_url = "https://username.github.io/repository-name/"

# The site title (required for SSG)
title = ""

# The scraps directory path relative to this Config.toml (optional, default=scraps)
scraps_dir = "scraps"

# The site language (compliant with iso639-1, default=en)
lang_code = "en"

# The site description (optional)
description = ""

# The site favicon in the form of png file URL (optional)
favicon = ""

# The site timezone (optional, default=UTC)
timezone = "UTC"

# The site color scheme (optional, default=os_setting, choices=os_setting or only_light or only_dark)
color_scheme = "os_setting"

# Build a search index with the Fuse JSON and display search UI (optional, default=true, choices=true or false)
build_search_index = true

# Scraps sort key choice on index page (optional, default=committed_date, choices=committed_date or linked_count)
sort_key = "committed_date"

# Scraps pagination on index page(optional, default=no pagination)
paginate_by = 20
```
