#[[Emit/Static Site]]

`README.md` at the wiki root is special-cased: it is not a scrap, and it
renders at its own `about/index.html` page, linked from the sidebar next to
the site title. This keeps the home a designed listing whatever the README
grows into, while the file still renders directly on GitHub.

No configuration — the presence of `README.md` is the switch. Without one,
the sidebar shows no about link.
