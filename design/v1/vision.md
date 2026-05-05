# Scraps v1 Vision

> Status: working draft for v1. Lives under `design/v1/` while v1 is in flight; expected to migrate into `docs/Explanation/` after v1 release.

## Vision

**The Wiki-link doc compiler for the LLM era.**

Scraps treats documentation like a programming language. Wiki-linked markdown
becomes a typed source, compiling into a static site for readers and into JSON
any agent can shell into — turning Karpathy's *LLM Wiki* pattern into a typed,
queryable artifact. CLI primary with companion AI skills, fitting any editor
and any LLM agent.

The project's core bet is that markdown documentation is most useful when it is
**typed**: a `[[link]]` is a typed reference, a `#[[tag]]` is a typed
discriminator, a `ctx_path` is a bounded namespace. Type discipline, applied to
prose, is what lets the same source compile into a static site for human readers
and into structured JSON for LLM agents — without either consumer paying for the
other.

## Why Now

The 2026-04 landscape made this slot urgent. Karpathy's *LLM Wiki* gist crossed
16M views and OSS implementations are sprouting; Anthropic shipped CLAUDE.md /
auto-memory; DeepWiki productized code → wiki generation; Will Larson's
library-mcp showed the staff-engineer audience hand-rolling its own MCP wikis;
GitHub's Spec Kit framed "markdown as a programming language"; Datadog Labs
released **pup**, demonstrating the *CLI + bundled AI Skills* distribution
pattern; and Obsidian shipped an official CLI in 2026-02.

The space is in **fragmentation phase**. Before consolidation arrives, scraps
needs to lock in its design and vocabulary. The slot scraps targets is:

> human-curated, LLM-augmented, CLI-typed, multi-target compile.

Sandwiched between Karpathy's LLM-only path and the classic human-only wiki,
distinguished from Obsidian by being headless / single-binary / SSG-included,
and distinguished from `library-mcp` by being a typed compiler rather than a
weekend MCP wrapper.

## Architectural Principles

These are the architectural commitments v1 is built on. Each is the result of an
explicit design decision, not an emergent property.

### 1. Layered separation: CLI / Skill / Core libs / MCP

| Layer | Role | Surface |
| --- | --- | --- |
| **CLI** | Primary agent integration. Deterministic, typed primitives. Read-centric. `--json` is a first-class output. | `build`, `lint`, `get --json`, `search --json`, `log`, `todo --json` |
| **Skill** | LLM-driven workflows. Authoring (write) lives here. Bundled and distributed *pup*-style as official AI-facing docs. | ingest, query, file-back |
| **Core libs** | Trait-based plugin points. | `GitCommand`, future `MetadataProvider` |
| **MCP** | De-emphasized in narrative; code is kept. v1 does not deprecate the existing `scraps mcp serve` / `plugins/mcp-server/`; v2+ revisits. | (kept as-is) |

CLI + JSON is the agent-integration primary path because shell + JSON is the
half-century-old contract: every agent that can run a shell can use it, no MCP
client implementation required, no long-running process to manage, scriptable in
CI. MCP's only advantage over CLI is streaming / push, which scraps' read-shaped
surface does not need. The decision to *narratively de-emphasize* MCP without
*removing* it mirrors the SSG decision (see §6): keep the code, decentralize
the story.

### 2. Per-file metadata is syntax — no frontmatter

Scraps takes the position that per-file metadata belongs in markdown syntax
(`#[[tag]]`, `[[link]]`, `![[embed]]`), not in a YAML/TOML frontmatter block:

- **No native frontmatter.** A read-only Obsidian-style adapter is allowed
  (passthrough only, no semantic interpretation; see #515) but scraps defines
  no frontmatter fields of its own.
- **No page-level alias.** Filesystem identity is the unique key; if you need
  another name, create another scrap.
- **Author / mtime / etc. is delegated to git** via the `GitCommand` trait —
  scraps does not store these per-file.
- **No auto-generated metadata footer in HTML output.** Templates surface
  whatever the author chooses.

This is a v0 → v1 break: the previous template + frontmatter machinery was
removed in #490. The reason is dual-authoring cost: AI agents (`scraps-writer`,
planned `scraps-agent`) generate scraps via LLMs; static templates plus
frontmatter would mean maintaining two authoring paths whose semantics drift
apart.

### 3. Tags are `#[[tag]]` (discriminator-leading)

`#[[tag]]` is preferred over `[[#tag]]` for two reasons. First, `[[#heading]]`
is needed for heading references (#472), so the leading `#` inside the brackets
is reserved. Second, the `discriminator + payload` shape (`:keyword`,
`@decorator`, `#[derive]`) is the PL-conventional way to mark a typed atom.

- Tags and scraps live in **separate namespaces**.
- A `[[name]]` that resolves to nothing is a **lint error** (the v0 implicit-tag
  fallback is gone).
- Nested tags `#[[a/b/c]]` get Logseq-style auto-aggregation, with **max depth
  3**.
- The old `singleton_tag` rule is removed; typo detection moves to `broken_link`.

### 4. Context is the filesystem

`scraps/a/b/title.md` becomes `ScrapKey { title, ctx: ["a", "b"] }`. Concretely:

- `ctx` depth is bounded at **3** (configurable via `.scraps.toml`, no CLI
  flag).
- Short-form `[[name]]` resolution searches by title; **ambiguity is an error**,
  Java-import style.
- Compared to Obsidian: Obsidian's folders are disambiguators on top of a
  flat, name-keyed identity; scraps treats `ctx_path` as part of the identity
  itself, with a bounded depth so the type does not balloon.

### 5. Git is plugin-shaped

`GitCommand` is a trait — git is a plugin point, not a hard dependency.

- `--git` is opt-in for build-time metadata that depends on git
  (`commited_ts`, future author).
- Commands that intrinsically require git (e.g. `scraps log`) need no flag and
  error if git is missing.
- Author / blame / additional metadata is out of v1 scope but is reachable by
  extending the trait without breaking the core.

### 6. Lint is the single wiki-health entry point

`scraps lint` is *the* health surface — every wiki-health rule lives there, and
no separate `wiki-doctor` or `wiki-check` command is added.

- Rule names suffix the dependency they rely on (`stale_by_git`, future
  `*_by_llm`), so the implementation graph is legible from the rule list.
- Default rules are **graph-mechanical only**; rules with external dependencies
  are opt-in via `--rule <name>`.
- v1 lint set: `dead_end`, `lonely`, `overlinking`, `self_link`, `broken_link`,
  `broken_heading_ref`, `stale_by_git`.

### 7. Wiki-wide aggregation is its own command; single-scrap introspection is `get --json`

A clean split that keeps the CLI surface small:

- **Wiki-wide** aggregations (`scraps todo`, `scraps log`, etc.) are
  standalone commands because their value is in the cross-cutting view.
- **Single-scrap** introspection (`headings`, `code_blocks`, future structured
  fields) is exposed as a field on `scraps get --json`, not a new top-level
  command.

### 8. Lean core, heavy deps as plugins

The core scraps binary stays single-file Rust with no heavy deps. The
performance budget is **build ≤ 3 seconds** (validated in CI on
[boykush/wiki](https://github.com/boykush/wiki)). Heavy deps that *would* be
useful — Lindera + ipadic for Japanese tokenization (50–100MB), LLMs,
embeddings — are plugin candidates. `missing_ref` lint depends on a Japanese
tokenizer and is therefore deferred to a plugin.

### 9. Karpathy's LLM Wiki is a narrative anchor, not a template

scraps shares Karpathy's *Ingest / Query / Lint* primitives and the
"persistent, compounding artifact" framing — but does not adopt his `log.md`
sidecar (git history covers it) and trades his implicit cross-references for
an explicit `[[link]]` graph with stronger type discipline. The *file-back*
concept is adopted as a separate skill.

### 10. No migration tooling, no archive feature

Migration tools are out of v1 scope: users read the v1 spec and run their own
LLM agent against their wiki. Archive is not a feature — `git rm` plus
`scraps log --deleted` (when implemented) is the pattern. This deliberately
keeps lint, search, build, and resolution free of `if archived` special cases.

### 11. Templates are removed

`scraps template generate / list` and `src/usecase/template/` are gone.
`modules/libs/src/markdown/frontmatter.rs` is gone (templates were its only
consumer). `TemplateError` and `templates_dir` are removed from `PathResolver`
and `TempScrapProject`. The replacement is the AI-skill authoring path
(`scraps-writer`, planned `scraps-agent`). PR #490.

## Build Target Philosophy

> SSG is one build target among N. It is not the center of v1.

The pipeline shape is:

```
markdown sources  →  scraps native IR (typed graph)  →  emitters
```

v1 ships two first-class emitters:

- **HTML** — the existing static-site output, kept as-is.
- **CLI JSON queries** — `scraps get`, `search`, `links`, `backlinks`, `todo`,
  etc. with `--json`. This is the **agent-integration primary**.

Future-candidate emitters (not v1 scope):

- `llms.txt`-style packed corpus
- Graph export (JSON / DOT)
- Pandoc AST as a *gateway* into the PDF / EPUB / docx chains

A note on Pandoc: scraps does **not** adopt Pandoc AST as the internal IR.
`[[wiki-link]]`, `#[[tag]]`, `![[embed]]`, `[[name#heading]]`, and `ctx_path`
are not first-class nodes in Pandoc AST and the type discipline they encode
cannot be expressed there. Pandoc AST is welcome as one of N emit targets, not
as the spine.

This is why the v1 README and pitch say *compiler* and *compile* rather than
*build tool* or *SSG*: those latter terms over-index on HTML.

## Config Discovery: the mise pattern

`.scraps.toml` declares a wiki by its presence — its directory **is** the wiki
root. There is no explicit `scraps_dir` config any more (#509). Every `*.md`
under that directory is a scrap; `ctx` is the relative path.

- **Decentralized.** Each `.scraps.toml` is its own independent wiki. A repo
  can host many of them; they do not cross-link, they each build to their own
  `output_dir`.
- **No upward walk.** Unlike mise (which composes multiple `.tool-versions`
  / `.mise.toml` up the tree), scraps does not search upward — the compose
  motivation does not exist for documentation.
- **`-C` / `--directory`** replaces `-p` / `--path` (#510), aligning with
  `git -C`, `make -C`, `pnpm -C`. The old flag stays as a deprecated alias for
  one release.
- **`output_dir`** defaults to `_site/` instead of `public/` (#508), aligning
  with Jekyll / Hakyll and avoiding ctx collisions with a `public/` directory
  in the source tree.
- `README.md → index.html` continues to be a special case.

Yarn-style workspaces (a root config plus an explicit member list) are out of
v1 scope; the decentralized mise pattern handles the cases v1 needs, and any
workspace orchestration is a v1.1+ question.

## Out of Scope (v1)

The following are intentionally not v1, with the reasoning preserved here so
that future contributors can re-evaluate from the right premises:

- **Templates / frontmatter.** Removed. Authoring goes through skills (#490).
- **Per-file `author` metadata.** Git plus a `GitCommand` extension covers it.
- **`missing_ref` lint.** Requires a Japanese tokenizer; plugin candidate.
- **`co_mention` lint.** Subsumed by `lookup_scrap_backlinks`.
- **`incomplete` lint.** Requires LLM judgment; skill candidate.
- **Block reference `[[name^id]]`.** Obsidian-specific; deferred.
- **Tables / callouts / image extraction.** Add as `scraps get --json` fields
  if needed.
- **Remote MCP / SaaS.** v1 stays local-first; MCP itself is de-emphasized.
- **Inline Dataview-style fields.** Reintroduces per-file metadata risk.
- **Mermaid / graph integration.** Heavy parsing; deferred.
- **Yarn-style workspaces.** Decentralized mise pattern wins for v1; revisit in
  v1.1+.
- **Migration tooling.** Users LLM-migrate their own wiki against the v1 spec.
- **Archive feature.** `git rm` + future `scraps log --deleted` is the pattern.
- **MCP server core-ization.** v1 keeps the existing implementation but does
  not promote it; CLI + JSON is the agent-integration primary.

## References

### Lineage

- **Write the Docs** community — *Docs as Code* discipline, treating
  documentation as engineered artifacts.
- **Andrej Karpathy — "LLM Wiki"** —
  https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f.
  Source of the *Ingest / Query / Lint* framing and the "persistent,
  compounding artifact" claim that scraps' typed graph operationalizes.

### Distribution pattern

- **Datadog Labs — pup** — https://github.com/datadog-labs/pup and the
  companion https://github.com/datadog-labs/agent-skills. Prior art for the
  *CLI primary + bundled AI skills* distribution pattern that scraps adopts
  via `scraps-writer` and the planned `scraps-agent`.
- **agents.md spec** — https://agents.md/. The interop layer that makes the
  pup-style distribution work across Claude Code, Codex CLI, Gemini CLI,
  Cursor, Windsurf, OpenCode, and similar clients.
