---
name: ingest
description: Create a new scrap from a source (prompt, URL, or arbitrary markdown), update cross-links in related scraps, and run a sanity check. Use this when the user wants to add a scrap, summarize an article, save synthesized content to the wiki, or file back a Q&A result.
allowed-tools: Read, Write, Edit, Bash, WebFetch, WebSearch, Glob, Grep
user-invocable: true
argument-hint: [source]
---

# Ingest

Add a new scrap to the wiki and integrate it into the existing graph.

Implements Karpathy's *Ingest* primitive for Scraps: read a source, draft a new scrap, update related scraps with cross-links, and sanity-check the result. A single ingest typically touches the new scrap plus 1–5 existing scraps.

## When to use

- "Write a scrap about X" / "Add a scrap on Y" (prompt source)
- "Summarize this article" / a pasted URL (web source)
- "Save this answer as a scrap" / file-back from a query (markdown source)

## Source types

| Source | Provided as | First step |
| --- | --- | --- |
| prompt | user's topic, instruction, or ready-to-use content (e.g., a `/query` synthesis) | if the input is complete content, use as-is; otherwise gather context (search related scraps) and ask one clarifying question only when scope is unclear |
| URL | pasted link | `WebFetch <url>`, extract title and key content |
| term | an atomic term surfaced in discussion (often unknown until just now) | treat the term itself as the title; skip clarifying questions |

## Workflow

1. **Identify source and topic**
   - URL: `WebFetch` → use OGP / heading title as initial title
   - prompt: if the input is complete content (e.g., a `/query` synthesis), use as-is; if it's a topic, ask one clarifying question only when scope is unclear
   - term: the term is the title; do not ask clarifying questions (the discussion already established scope). Proceed to step 2 with the term as the search keyword.

2. **Research existing wiki state**
   - `scraps search "<keyword>" --json` to find related scraps
   - `scraps tag list --json` to find relevant tags
   - Read 3–8 of the most related scraps via `scraps get "<title>" --json`
   - Use `scraps get "<title>" --json headings` first when only the outline is needed

3. **Decide title and ctx**
   - Pick a clear, atomic title
   - If the title collides with an existing scrap, add a context folder: `scraps/<ctx>/<title>.md`
   - ctx depth ≤ 3

4. **Decide max-lines (familiarity heuristic)**
   - Count related scraps from step 2 (low: 0–5, medium: 6–15, high: 16+)
   - Low → 5–7 lines (protect working memory)
   - Medium → 10–12 lines (schema is forming, more detail welcome)
   - High → 5–7 lines (avoid redundant explanation; prefer link-rich brevity)
   - Skip this step if the user specified `--max-lines` explicitly (CLI flag is authoritative)
   - If the user signals depth in conversation ("go deeper" / "longer" / "厚めに" etc.), bump one band upward; "shorter" / "ライトに" etc. bumps downward. Conversation cue overrides the heuristic but stays below the explicit `--max-lines` flag.

5. **Draft the scrap**
   - Plain Markdown with `[[link]]` for references and `#[[tag]]` for tags
   - Include source autolink if URL: `<https://...>`
   - Stay within max-lines

6. **Cross-link update** (Karpathy's "update related entity and concept pages")
   - For each of the 1–5 most related existing scraps, search the body for plain-text mentions of the new title; convert each mention to `[[new title]]` when the link direction protocol allows. Do not invent new mentions where none exist.
   - **Link direction protocol**: links always flow concrete → abstract. The new scrap can be at either end:
     - new scrap is concrete (Book, Project, Tool) → edit the NEW scrap to link out to the existing abstract scraps it depends on
     - new scrap is abstract (a concept/topic) → edit the EXISTING concrete scraps that mention it to link in: convert their plain-text mentions to `[[new title]]`
     - existing abstract scrap → do NOT add a reference back to a new concrete scrap (anti-pattern)
     - sibling scraps may link only when the relation is direct and asymmetric
   - **Anti-patterns**: bidirectional links between abstract↔concrete, "for completeness" additions, mention-based reflexive linking
   - Backlinks are auto-computed by Scraps; explicit reverse links are redundant

7. **Post-write sanity check**
   - `scraps lint --rule broken-link` (purpose: confirm the new scrap introduced no broken references)
   - For non-trivial violations, hand off to the `lint-rule-handler` agent

## Wiki-link syntax

| Syntax | Meaning |
| --- | --- |
| `[[Title]]` | normal link |
| `[[Title|Display]]` | alias |
| `[[Ctx/Title]]` | context-disambiguated link |
| `[[Title#Heading]]` | heading reference |
| `#[[Tag]]` | tag |
| `<https://...>` | autolink (renders as OGP card) |

## CLI used

- `scraps search <query> --json`
- `scraps get <title> [--ctx <ctx>] [--heading <heading>] --json [fields]`
- `scraps tag list --json`
- `scraps tag backlinks <tag> --json`
- `scraps lint --rule <rule>`

Full command details: `scraps <cmd> --help`.

## Further reading

- Scraps docs: <https://boykush.github.io/scraps/>
- Karpathy's *Ingest / Query / Lint* framing: <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>
- Composition with other primitives, dialogue / catch-up patterns, and primitive boundaries: see the `scraps-llm-wiki-schema` agent.
