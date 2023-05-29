## Installation
```bash
cargo install scraps
```

## Init project
```bash
mkdir -p your-scraps-project/scraps
cd your-scraps-project
touch Config.toml
```
※ Soon, we will provide a `scraps init` command instead.

## Configuration
Please set the required fields.
```toml:Config.toml
title = "your scraps project"
```

See [[Configuration]] for other config.

## Write scraps
Write markdown files under `/scraps` dir.

```
❯ tree scraps
scraps
├── Getting Started.md
└── Scraps.md
```

See [[Scraps]] for scraps notation in markdown files.

## Build static site

```
scraps build
```

The output is as follows.

```
❯ tree public
public
├── Getting Started.html
├── Scraps.html
├── index.html
└── main.css
```

## Deploy pages
See [[Deploy]].
