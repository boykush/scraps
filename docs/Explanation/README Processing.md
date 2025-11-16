#[[Markdown]]

In Scraps, the `scraps/README.md` file is automatically converted to HTML and included in the static site's top page ( `public/index.html` ).

For Markdown syntax, please refer to [[Reference/CommonMark]].

## Limitations

When using autolink syntax in `scraps/README.md`, the OGP card described in [[Reference/Autolink]] will not be displayed.
URLs will be displayed as normal links.

---

#### In

```markdown
<https://example.com>
```

#### Out

<a href="https://example.com">https://example.com</a>
