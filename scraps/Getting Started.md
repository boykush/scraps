## Installation
```bash
cargo install scraps
```

### mac OS
```
brew install boykush/tap/scraps
```

## Init project
`git` command is required for features
```bash
scraps init your-scraps-project
cd your-scraps-project
```

The output result will look like this

```bash
❯ tree -a -L 1
.
├── .git
├── .gitignore
├── Config.toml
└── scraps
```

## Configuration
Edit `Config.toml` to settings.

```toml:Config.toml
title = "your scraps project"
```

See [[Configuration]] for other config.

## Write scraps
Write markdown files under `/scraps` dir.

```bash
❯ tree scraps
scraps
├── Getting Started.md
└── Scraps.md
```

See [[Scraps]] for scraps notation in markdown files.

## Build static site

```bash
scraps build
```

The output will be a slugged html file as follows.

```bash
❯ tree public
public
├── getting-started.html
├── scraps.html
├── index.html
└── main.css
```

## Deploy pages
See [[Deploy]].
