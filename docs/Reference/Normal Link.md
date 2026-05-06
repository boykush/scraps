#[[Wiki-Links]] #[[Markdown]]

Specifying the name of the markdown file with a notation such as `[[Link]]` will generate a wiki-like internal link.

For example, if you have the following set of files.

```bash
❯ tree
.
├── Overview.md
└── Usage.md
```

Fill in the target file name in `Overview.md` as follows to generate the link.

```markdown:Overview.md
See [[Usage]] for detail.
```
