---
name: query
description: Answer questions about the wiki by searching, reading, and synthesizing relevant scraps with `[[Title]]` citations. Use this when the user wants to query the wiki, ask what they have written about a topic, compare scraps, find related notes, or pull a synthesized overview from existing scraps.
allowed-tools: Read, Bash, Glob, Grep
user-invocable: true
argument-hint: [question]
---

# Query

Answer a question against the wiki and return a citation-rich synthesis.

Implements Karpathy's *Query* primitive for Scraps: search the wiki, read relevant scraps, and synthesize an answer that references its sources by `[[Title]]`. Citations make the answer auditable and ready to be filed back as a new scrap if the user chooses.

## When to use

- "What do I have on X?" / "What did I write about Y?"
- "Compare X and Y across my notes"
- "Summarize what I know about Z"
- "Find scraps related to W"

## Workflow

1. **Understand the question**
   - Identify the core topic and any constraints (time range, ctx folder, tag, etc.)
   - If the question is ambiguous, ask one clarifying question before searching

2. **Search (broad → narrow)**
   - `scraps search "<keyword>" --json` for each main keyword
   - Try multiple phrasings if the first search returns few results
   - For tag-driven questions: `scraps tag backlinks "<tag>" --json`

3. **Select candidates**
   - From search results, pick the 5–15 most relevant scraps
   - Prefer scraps that span the question (different ctx, different tags) over many near-duplicates

4. **Read selected scraps**
   - `scraps get "<title>" [--ctx <ctx>] --json` for each candidate
   - For graph-shaped questions, also use:
     - `scraps links "<title>" --json` (outbound)
     - `scraps backlinks "<title>" --json` (inbound)

5. **Synthesize with citations**
   - Write the answer in plain Markdown
   - Cite every claim that comes from a scrap as `[[Title]]` (or `[[Ctx/Title]]` when needed)
   - Do not invent information beyond what the scraps and the user's question support
   - Choose the output shape that fits the question:
     - prose paragraph for narrative questions
     - GFM table for comparisons
     - bullet list for enumerations
     - mermaid diagram for relationships

6. **Stop**
   - Do not auto-offer to save the answer. If the user wants the synthesis saved as a new scrap, they invoke the `ingest` skill on the answer markdown.

## Citation rules

- Every non-trivial claim should trace to a `[[Title]]` citation.
- If a claim has no source in the wiki, mark it as outside the wiki ("not in the current scraps"); do not bluff.
- Use the exact `title` value returned by `scraps search --json` / `scraps get --json` so the citation resolves cleanly.
- For ctx-disambiguated scraps, use `[[Ctx/Title]]`.

## CLI used

- `scraps search <query> [--logic and|or] --json`
- `scraps get <title> [--ctx <ctx>] --json`
- `scraps links <title> [--ctx <ctx>] --json`
- `scraps backlinks <title> [--ctx <ctx>] --json`
- `scraps tag list --json`
- `scraps tag backlinks <tag> --json`
- `scraps todo [--status open|done|deferred|all] --json`

Full command details: `scraps <cmd> --help`.

## Further reading

- Scraps docs: <https://boykush.github.io/scraps/>
- Karpathy's *Ingest / Query / Lint* framing: <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>
