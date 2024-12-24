#[[CLI Usage]]

```bash
❯ scraps build
```

This command builds the Markdown files under the `/scraps` directory and generates static site files.

#### Markdown files
```bash
❯ tree scraps
scraps
├── Overview.md
└── Markdown Syntax.md
```

#### Generated files
The output will be a slugged html file as follows.
```bash
❯ tree public
public
├── overview.html
├── markdown-syntax.html
├── index.html
└── main.css
```

The next step is [[Serve]].