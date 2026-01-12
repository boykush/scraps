To use Scraps as a static site generator (SSG), you need to configure the
`base_url` and `title` fields in your `Config.toml` file.

## Step 1: Edit base_url

Replace the placeholder URL with your actual site URL:

```toml
base_url = "https://yourusername.github.io/your-repository/"
```

## Step 2: Set your site title

Add your desired site title:

```toml
title = "My Knowledge Base"
```

## Optional Configuration

The `base_url` and `title` variables are required when using Scraps as a static
site generator (SSG). Other commands like `tag`, `mcp`, and `template` can work
without these fields. All other configuration variables are optional.

See [[Reference/Configuration]] for all available configuration variables and
their default values.
