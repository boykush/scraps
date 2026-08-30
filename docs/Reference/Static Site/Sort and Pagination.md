#[[Emit/Static Site]]

Every sort view is always generated — a sort order is a URL, not a config:

- `/` — updated view (committed date, descending). This is the home.
- `/backlinks/` — most linked first.
- `/titles/` — a one-page title index grouped by the title's initial:
  gojuon rows (voiced, semi-voiced and small kana fold onto their base row,
  and katakana folds to hiragana, so デ lands on た), then `A`–`Z`, then a
  `漢字` group, then `#` for anything else. Groups with no scraps are not
  rendered.

The `/` and `/backlinks/` views share the same pagination, configured by
`paginate_by` under `[ssg]` in [[Reference/Configuration]].
