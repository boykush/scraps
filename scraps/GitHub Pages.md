#[[Deployment]]

Custom actions are available to deploy Scraps to Github Pages.

[scraps-deploy-action](https://github.com/boykush/scraps-deploy-action)

### YAML file
Prepare a yaml file under `.github/workflows/` like this

```yaml
name: Deploy scraps github pages
on: 
  push:
    branches:
      - main
    paths:
      - 'scraps/**'
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: checkout
        uses: actions/checkout@v5
        with:
          fetch-depth: 0 # For scraps git commited date
      - name: build_and_deploy
        uses: boykush/scraps-deploy-action@v2
        env:
          # Target branch
          PAGES_BRANCH: gh-pages
          TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### GitHub settings
Set up GitHub Pages for the repository.

`Build and deployment` parameter as follows.
- Source: `Deploy from a branch`
- Branch: `gh-pages`
