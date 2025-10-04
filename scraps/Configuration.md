Configuration is managed by `Config.toml` in the Scraps project.

## Quick Setup Guide

To get started with your Scraps site, you need to edit at least two required fields in your `Config.toml` file:

### Step 1: Edit base_url
Replace the placeholder URL with your actual site URL:
```toml
base_url = "https://yourusername.github.io/your-repository/"
```

### Step 2: Set your site title
Add your desired site title:
```toml
title = "My Knowledge Base"
```

## Configuration Variables

Only the `base_url` and `title` variables are required. Everything else is optional. All configuration variables used by Scraps and their default values are listed below.

```toml:Config.toml
# The site base url
base_url = "https://username.github.io/repository-name/"

# The site title
title = ""

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

# Scraps pagination on index page (optional, default=no pagination)
paginate_by = 20
```