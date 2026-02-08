#[[CLI]]

```bash
❯ scraps lint
```

This command checks your scraps for potential issues such as implicit tags.

When a `[[link]]` references a title that does not match any existing scrap, it is implicitly treated as a tag. The lint command warns about these cases and recommends using `#[[tag]]` to make the intent explicit.

Writing `#[[tag]]` suppresses the warning, signaling that the tag is intentional. See [[Reference/Tag Link]] for details on the tag syntax.

## Examples

```bash
# Lint all scraps
❯ scraps lint

# Lint scraps in a specific project
❯ scraps lint --path /path/to/project
```

## Exit code

The command exits with code `1` if any warnings are found, making it suitable for CI pipelines.
