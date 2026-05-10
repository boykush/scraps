#[[CLI]]

This guide gets you from an empty directory to a small Scraps wiki that can
be built as a static site and queried from the CLI.

For the bigger picture of what Scraps is and why, see
[[Explanation/What is Scraps?]].

## Setup

1. **Install Scraps** — follow [[Tutorial/Installation]].

2. **Initialize a project** — create a directory and initialize it:

   ```bash
   ❯ mkdir my-knowledge-base
   ❯ cd my-knowledge-base
   ❯ scraps init
   ```

   This writes a `.scraps.toml` to the current directory. The directory
   containing it becomes the wiki root.

3. **Configure the project** — open `.scraps.toml` and set `[ssg]` `base_url`
   and `title`. See [[Reference/Configuration#ssg-section]] for the full
   schema.

## Authoring

1. **Write Markdown files** next to `.scraps.toml` or in folders under it.
   Standard CommonMark and GitHub-flavored Markdown are supported — see
   [[Reference/Markdown Support]].

2. **Connect scraps with wiki-links.** The full notation is in
   [[Reference/Wiki-link Notation]]; the most common forms are:

   - `[[Page Name]]` — normal link
   - `[[Page Name|Display]]` — alias
   - `[[Folder/Page Name]]` — context-qualified
   - `[[Page Name#Heading]]` — heading reference
   - `![[Page Name]]` — embed another scrap inline
   - `#[[Topic]]` — tag (separate namespace from scraps)
   - `#[[Area/Sub]]` — nested tag (auto-aggregated)

## Build and preview

```bash
❯ scraps build      # write _site/
❯ scraps serve      # serve at http://127.0.0.1:1112
```

The output structure, `README.md` handling, and search index are documented
in [[Reference/Static Site]]. For deploying, see
[[How-to/Deploy to GitHub Pages]].

## Lint

`scraps lint` checks wiki health: dead-end scraps, broken links, broken
heading references, repeated links, and more. Rules are documented in
[[Reference/Lint Rules]].

```bash
❯ scraps lint
```

## AI integration

Scraps is CLI-first for AI agents. Any assistant that can run shell commands
can query the wiki:

```bash
❯ scraps search "query" --json
❯ scraps get "Page Name" --json
❯ scraps get "Page Name" --heading "Section" --json body
❯ scraps backlinks "Page Name" --json
❯ scraps todo --json
```

`scraps get --json` returns `title`, `ctx`, and `body` by default. It can also
project fields such as `headings` or `code_blocks`, and `--heading` narrows
the read to one section.

For Claude Code users there is also an official skills bundle. See
[[How-to/Integrate with AI Assistants]] for both paths.
