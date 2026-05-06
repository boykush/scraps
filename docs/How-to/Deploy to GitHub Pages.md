#[[Deployment]] #[[Emit/Static Site]]

Deploy a Scraps site to GitHub Pages using GitHub Actions. The build output
(`_site/` by default; configurable in
[[Reference/Configuration#root-level]]) is uploaded as a Pages artifact and
published via the official `actions/deploy-pages` action — no `gh-pages`
branch required.

## GitHub settings

Set up GitHub Pages for the repository.

`Build and deployment` parameter as follows:

- Source: `GitHub Actions`

## Workflow file

Prepare a YAML file under `.github/workflows/` like this:

```yaml
name: Deploy scraps github pages
on:
  push:
    branches:
      - main
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v6
        with:
          fetch-depth: 0 # For scraps git committed date

      - name: Install Scraps
        uses: jdx/mise-action@v2
        with:
          mise_toml: |
            [tools]
            "github:boykush/scraps" = "v0.33.0"

      - name: Build
        run: scraps build

      - name: Configure Pages
        uses: actions/configure-pages@v5

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: _site

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

Scraps is installed via mise using the `github:boykush/scraps` backend, which
fetches a binary from GitHub Releases. Pin a released tag so deploys stay
reproducible.

<https://mise.jdx.dev/>

If you already maintain a `mise.toml` in the repository, you can omit the
`mise_toml` input and `jdx/mise-action` will pick it up automatically.

If your `output_dir` differs from `_site/`, update the `path:` in the
`upload-pages-artifact` step to match.
