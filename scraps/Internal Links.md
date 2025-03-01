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

## Tags as nonexistent links
If there is no scraps with the specified title, such as #[[Content]], then it will be a tag.

Tags are lined up on the index page, and the link is to a page with a list of scraps that have the tag.

## Alias by display text
Internal links allow you to use different display text as aliases.

Writing `[[Content|here]]` creates a link that displays as [[Content|here]] but links to the "Content" page.