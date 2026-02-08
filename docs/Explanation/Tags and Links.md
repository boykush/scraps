#[[Wiki-Links]]

This document explains the design philosophy behind how Scraps distinguishes tags from links.

## The problem with implicit tags

In the original design, any `[[link]]` that did not match an existing scrap was automatically treated as a tag. This made it impossible to distinguish between intentional tags and typos or broken references. There was no way to lint for mistakes, and the writer's intent was invisible.

## The `#` prefix: minimal switching cost

Scraps introduces `#[[tag]]` notation to explicitly mark a wiki-link as a tag:

- `[[x]]` — Always a link. Expects a corresponding scrap to exist.
- `#[[x]]` — Always a tag. Treated as a tag regardless of whether a scrap exists.

The key insight is that switching between a tag and a link requires only adding or removing `#`. To promote a tag to a scrap, remove the `#` and create the scrap file. To demote a link to a tag, add `#`. This minimal switching cost keeps the workflow lightweight.

## Why not `#tag`?

Many tools such as Obsidian and Foam use `#tag` (without wiki-link brackets). Scraps deliberately chose `#[[tag]]` for the following reasons:

- **Consistency**: Tags use the same `[[wiki-link]]` notation as everything else in Scraps
- **Multi-word support**: `#[[Domain Driven Design]]` works naturally without special escaping
- **No ambiguity**: `#` followed by `[[` is unambiguous, while `#tag` requires rules for where the tag name ends

## Hierarchical organization

Scraps does not introduce hierarchical tags like `#parent/child`. Instead, use [[Reference/Context Link]] for hierarchical organization. Tags remain flat labels for categorization.

## Future direction

In a future release, `#[[tag]]` will become the only way to define tags. Plain `[[link]]` will always be treated strictly as a link, and implicit tag detection will be removed.
