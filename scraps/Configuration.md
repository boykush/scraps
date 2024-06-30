Configuration is managed by `Config.toml` in the Scraps project.

Only the title variable is required. Everything else is optional. All configuration variables used by Scraps and their default values are listed below.

```toml:Config.toml
# The site title
title = ""

# The site description (optional)
description = ""

# The site favicon in the form of png file URL (optional)
favicon = ""

# The site timezone (optional, default=UTC)
timezone = "UTC"

# Scraps sort key choice (optional, default=committed_date, choices=committed_date or linked_count)
sort_key = "committed_date"

# Scraps pagination on index page(optional)
paginate_by = 20
```
