#[[Notation/Markdown]]

Bare URLs wrapped in angle brackets render as OGP cards on the static
site, fetching the target page's Open Graph metadata at build time.

```markdown
<https://github.com/boykush/scraps>
```

The example above renders as a live OGP card on this page:

<https://github.com/boykush/scraps>

Use autolinks deliberately: the card is heavy compared to an inline link.
Reach for `<…>` when the link is a "you should look at this" pointer
(canonical source, prior art, recipe target). Use `[text](url)` for inline
references that the reader skims past.

The wiki root `README.md` falls back to plain links — see
[[Reference/Static Site/README and Index]] for the limitation.
