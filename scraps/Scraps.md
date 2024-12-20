#[[Content]]

## Internal links like wiki
Specifying the name of the markdown file with a notation such as `[[Link]]` will generate a wiki-like internal link.

For example, if you have the following set of files.
```bash
❯ tree scraps
scraps
├── Overview.md
└── Usage.md
```

Fill in the file name in the `scraps` directory in `Overview.md` as follows to generate the link.
```markdown:Overview.md
See [[Usage]] for detail.
```