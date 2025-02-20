The template feature can be used by preparing Markdown files as templates under the `templates/` directory.

This feature is implemented based on [Tera](https://github.com/Keats/tera), a template engine written in Rust. 

In templare feature of Scraps, you can mainly use Tera syntax and [built-in functions](https://keats.github.io/tera/docs/#built-in-functions).

For more details, please refer to the [Tera documentation](https://keats.github.io/tera/docs/).

## Scraps extension

In addition to Tera's syntax and built-ins, custom extensions can be implemented. Here are the custom extensions currently available in Scraps.

### Variables

#### timezone
You can use the `timezone` specified in `Config.toml`.

```
{{ timezone }}
```

As an example, use it as the `timezone` argument in the [date](https://keats.github.io/tera/docs/#date) filter.
```
{{ now() | date(timezone=timezone) }}
```

Please submit your extension requests to the [Issue](https://github.com/boykush/scraps/issues/new?template=enhancement-feature-template.md).

## Others
For samples of templates using Tera, please refer to [[Scraps templates]].