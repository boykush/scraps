#[[Templates]]

## Daily note
Utilizes Tera's standard [now()](https://keats.github.io/tera/docs/#now) function, [date](https://keats.github.io/tera/docs/#date) filter, and Scraps' custom `timezone` variable.

```md
+++
title = "{{ now() | date(timezone=timezone) }}"
+++
```

## Arguments by environment variables
Using the [get_env()](https://keats.github.io/tera/docs/#get-env) function, you can write templates that customize arguments at the time of CLI execution.

```
+++
title = "[Book] {{ get_env(name="TITLE") }}"
+++

![cover]({{ get_env(name="COVER") }})
```

When execute generate
```
TITLE="Test-Driven Development By Example" COVER="https://m.media-amazon.com/images/I/71I1GcjT-IL._SY522_.jpg" scraps template generate book
```