You can find the latest version on [GitHub Releases](https://github.com/boykush/scraps/releases).

## Requirements

The `git` command is required for features.

### Cargo

```bash
❯ cargo install scraps
```

### macOS / Linux (Homebrew)

```bash
❯ brew install boykush/tap/scraps
```

### GitHub Releases

Download the binary for your platform and place it in your PATH:

```bash
# macOS (Apple Silicon)
❯ curl -sL https://github.com/boykush/scraps/releases/latest/download/scraps-aarch64-apple-darwin.tar.gz | tar xz

# macOS (Intel)
❯ curl -sL https://github.com/boykush/scraps/releases/latest/download/scraps-x86_64-apple-darwin.tar.gz | tar xz

# Linux (x86_64)
❯ curl -sL https://github.com/boykush/scraps/releases/latest/download/scraps-x86_64-unknown-linux-gnu.tar.gz | tar xz

# Linux (ARM64)
❯ curl -sL https://github.com/boykush/scraps/releases/latest/download/scraps-aarch64-unknown-linux-gnu.tar.gz | tar xz
```

Then move the binary to a directory in your PATH:

```bash
❯ sudo mv scraps /usr/local/bin/
```
