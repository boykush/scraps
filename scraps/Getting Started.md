## Installation
```bash
❯ cargo install scraps
```

### mac OS
```bash
❯ brew install boykush/tap/scraps
```

## Init project
`git` command is required for features
```bash
❯ scraps init your-scraps-project
❯ cd your-scraps-project
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

## Build a static site

```bash
❯ scraps build
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

## Debug server

```bash
❯ scraps serve
```

You can debug the build outputs by visiting `http://127.0.0.1:1112`

## Deploy pages
See [[Deploy]].
