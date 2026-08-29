# livt workspace

Product discovery workspace for Scraps, managed with
[livt](https://github.com/boykush/livt): opportunities, opportunity canvases,
and the ubiquitous language behind them.

## Language convention

Everything under this directory is intentionally written in **Japanese**, the
maintainer's working language for discovery, while the rest of the repository
stays in English. The site chrome follows suit (`lang: ja` in `livt.yaml`).

## Usage

The livt binary is pinned by `mise.toml` in this directory:

```bash
cd livt
mise exec -- livt serve   # preview at http://localhost:3000 with live reload
mise exec -- livt build   # static build into dist/ (gitignored)
```
