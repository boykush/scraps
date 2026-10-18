---
type: Reference
description: How a list orders its scraps through the query, and how many it shows at first.
---

#[[Emit/Static Site]]

Every list on the static site — the whole wiki at `/` and each tag's list
under `tags/` (see [[Reference/Static Site/Tag Pages]]) — can be ordered three
ways. The order is a query parameter, so every list offers all three and
switching keeps the list you are on:

| `sort` | Order |
|---|---|
| (none) | updated: committed date, newest first |
| `backlinks` | most linked first |
| `title` | by title, grouped by the title's initial with a jump bar |

The title order groups gojuon rows first (voiced, semi-voiced and small kana
fold onto their base row, and katakana folds to hiragana, so デ lands on た),
then `A`–`Z`, then a `漢字` group, then `#` for anything else. Groups with no
scraps are not shown.

A list page holds every scrap of its list and arranges them itself in the
browser. Without JavaScript it shows them all in the updated order.

`paginate_by` under `[ssg]` in [[Reference/Configuration]] sets how many rows
the updated and backlinks orders show before a *show more* button; the title
order always shows every row.

Committed dates come from git, which every build reads unless `--no-git`.
With `--no-git`, or outside a git repository, no scrap has a date and the
updated order has nothing to sort by.
