#[[Emit/Static Site]]

`README.md` at the wiki root is special-cased: it becomes `index.html`
instead of `scraps/readme.html`. This lets the wiki render correctly both
on the static site and directly on GitHub.

One limitation: [[Reference/Markdown/Autolink|autolinks]] inside the wiki
root `README.md` fall back to plain links — the OGP card is suppressed on
the index page.
