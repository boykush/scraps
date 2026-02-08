#[[Wiki-Links]] #[[Markdown]]

If there is no scrap with the specified title, such as #[[Markdown]], it becomes a tag.

Tags are displayed on the index page. Each tag links to a page that lists all scraps using that tag.

## Explicit tag notation

We recommend using `#[[Tag Name]]` to explicitly mark intentional tags. The [[Reference/Lint]] command warns about wiki-links that do not match any existing scrap and are not prefixed with `#`.

```markdown
#[[Rust]] #[[CLI]]

This scrap is tagged with Rust and CLI.
```

See [[Explanation/Tags and Links]] for the design philosophy behind this notation.
