## Github Pages
Custom actions are available to deploy Scraps to Github Pages.

[scraps-deploy-action](https://github.com/boykush/scraps-deploy-action)

### YAML file
Prepare a yaml file under `.github/workflows/` like this

```yaml
on: push
name: Build and deploy GH Pages
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: checkout
        uses: actions/checkout@v3
      - name: build_and_deploy
        uses: boykush/scraps-deploy-action@v0.1.2
        env:
          # Target branch
          PAGES_BRANCH: gh-pages
          # Provide personal access token
          TOKEN: ${{ secrets.TOKEN }}
```

### GitHub settings
#### 1. Set up GitHub Pages for the repository.

`Build and deployment` parameter as follows.
- Source: `Deploy from a branch`
- Branch: `gh-pages`

#### 2. Set `TOKEN` variable as Actions secrets.

