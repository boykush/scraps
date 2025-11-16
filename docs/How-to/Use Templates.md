#[[Templates]]

This document provides practical template samples that you can use immediately. Each sample includes detailed explanations and complete workflow examples.

## Daily Note

Creates a daily note with today's date as the title. This template utilizes Tera's standard [`now()`](https://keats.github.io/tera/docs/#now) function, [`date`](https://keats.github.io/tera/docs/#date) filter, and Scraps' custom `timezone` variable.

**Template file: `/templates/daily_note.md`**

```markdown
+++
title = "{{ now() | date(timezone=timezone) }}"
+++

# Daily Notes

## Today's Tasks
- [ ] 
```

**Usage:**

```bash
scraps template generate daily_note
```

This generates a scrap with the current date as title (e.g., "2024-01-15").

## Arguments by Environment Variables

Using the [`get_env()`](https://keats.github.io/tera/docs/#get-env) function, you can write templates that customize arguments at the time of CLI execution.

**Template file: `/templates/book.md`**

```markdown
+++
title = "[Book] {{ get_env(name="TITLE", default="") }}"
+++

![cover]({{ get_env(name="COVER", default="") }})
```

**Usage:**

```bash
TITLE="Test-Driven Development By Example" COVER="https://m.media-amazon.com/images/I/71I1GcjT-IL._SY522_.jpg" scraps template generate book
```
