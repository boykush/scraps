## Internal links like wiki
Specifying the name of the markdown file with a notation such as `[[Link]]` will generate a wiki-like internal link.

For example, if you have the following set of files.
```bash
❯ tree scraps
scraps
├── Overview.md
└── Usage.md
```
Generate a link by filling in `Overview.md` as follows.

```markdown:Overview.md
See [[Usage]] for detail.
```