#[[Integration]]

Currently, the most recommended LSP for editing Scraps is [markdown-oxide](https://github.com/Feel-ix-343/markdown-oxide).

markdown-oxide supports the following editing environments:

- Neovim
- VSCode
- Zed
- Helix

To match the current features provided by Scraps, place the following configuration file `.moxide.toml` under the `scraps/` directory and open the `scraps/` directory directly for a comfortable editing experience.

```toml
heading_completions = false
title_headings = false
tags_in_codeblocks = false
references_in_codeblocks = false
```

We are considering a feature to generate the LSP configuration file during the [[CLI/Init|init command]].