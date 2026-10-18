---
name: learn
description: Learn a source together with the user and record only what they understood as a new page in this Scraps wiki. Explains the source with a diagram anchored on the pages the wiki already has, checks comprehension with questions, then writes one page from the points that passed; when understanding is not reached, nothing is written. Use this whenever the user wants to learn or understand a URL, article, talk, or term, and whenever they want a term, tool, or product added as a new page, even if they do not ask for a lesson. To record an article or talk as a summary without dialogue, use digest.
argument-hint: "[source]"
---

# Learn

Help the user understand a source, then record only what they understood as one page in the wiki.

The wiki is a record of the user's understanding, not of what its sources say. When an AI drafts a summary and the user approves it, nobody can tell what was understood from what was merely read and nodded at. So before anything is written, the user answers for themselves. The goal is the user's understanding, and the page is what remains of it. A run that ends with nothing written, because understanding was not reached, has ended correctly.

Hold the dialogue and write the page in the user's language.

## Boundaries

- Write no file until the comprehension check is done.
- Write only what passed the check. Something you explained is not established until the user has shown they understand it.
- One run produces one page. If the source holds several concepts, pick the central one and name the rest as candidates for later runs.
- State as fact only what you verified in a primary source. Analogies and examples are fine as teaching aids, but build the questions and the page from what the source says.
- Do not write the user's answers for them. If you quietly fix a wrong answer and let it pass, the page records your understanding, not theirs.
- Change existing pages only by turning literal mentions the user approved into links.
- Do not widen the work into tidying or auditing the wiki.

## Asking the user

Several steps put choices to the user. How depends on the client, so decide it from the tools you actually have:

- When the client has a tool for asking multiple-choice questions (for example `AskUserQuestion` in Claude Code), use it. Its definition says how many questions a call takes, how many options a question takes, and whether several options can be picked; stay inside those.
- When there is no such tool, or what you need to ask does not fit it, ask in the message body: number the questions, letter the options, say how to reply (for example `1b 2d`), then stop and wait.

Either way the user can always answer in their own words. Take such an answer as it is rather than forcing it onto an option.

## 1. Find where the user stands

The existing pages are the list of what the user already understands. Reading them decides where the explanation starts and what it can skip.

1. Read the source and identify its central terms and claims. For a URL, fetch the primary source; given only a term, find a primary source for it. If only part of a source can be read (slide hosts such as Speaker Deck often yield just the title and abstract), work from what you could read and say what was missing instead of filling the gap by guessing.
2. Search with varied wording using `scraps search "<query>" --json`, then read the 3–8 most related pages by passing each hit's `key` back: `scraps get "<key>" --json`.
3. Split the central terms into those that already have a page and those that do not. A term has a page only when that thing is the page's subject. A page about a source (an article, a talk, one entry of a report or catalog) is not the page for the term in its title, even when the title is exactly the term; read the body to tell, and link such a page only when you mean that source. The former are known ground to build on. Fill in the latter only as far as this run needs, and keep them as candidates for later runs.
4. Settle the context: the part of the wiki the user is learning this source for, named by the existing page at its center. This is not a context folder. A source usually touches several parts of the wiki, and the user came for one of them. A gateway for agent traffic sits next to both the pages on agent protocols and the pages on cluster networking; a user who came for the first does not want to be quizzed on the second. Decide it in this order:
   - what the user said when asking, or earlier in the conversation
   - the pages the user added or changed lately, when the wiki is a git repository (`git log --since="1 month ago" --name-only --format=`)
   - if more than one context is still possible, ask which with one question before explaining, offering the possible contexts as options

   Do not settle it by which part has the most pages or backlinks. A hub the user filled in long ago would outweigh what they are learning now.
5. Decide the points that may be recorded: one gist saying what the thing is, and up to three connections saying how it relates to existing pages inside the context. Fewer is fine. A connection outside the context is not a point, however much room the source gives it; name it as a candidate for a later run. With no related pages, the gist alone is enough.

## 2. Explain

- Open by naming the context in one line, so the user can redirect before the explanation is spent on the wrong one.
- Start from terms that already have pages and place the new concept next to them. Use known terms as they are, marked `[[Title]]`, without explaining them again.
- Stay on the points. Do not summarize the source, and leave out detail the points do not need.
- Put one diagram at the center of the explanation. Relations, flows, layers, and comparisons are taken in faster at a glance than in prose. Draw existing pages as known landmarks and place the new concept among them. Use a diagram-rendering tool when the client has one (for example the visualize tool in the Claude desktop app); otherwise use a table or a text diagram.
- Give one concrete example before the definition. If a nearby page exists, show how the new concept differs from it.
- Skip what the user says they already know. Checking it is step 3's job.

After the explanation, stop and take questions. Move on to the check only when the user says to.

## 3. Check comprehension

Check the points with multiple-choice questions, then have the user state the gist in their own words.

A question that can be answered by remembering the wording of the explanation does not measure understanding. Give each question one of these shapes:

- apply the concept to a concrete case the explanation did not use
- tell it apart from a nearby concept
- ask how it relates to an existing page

Set every question inside the context: take the concrete cases from it and the nearby concepts from its pages. What the user asked about during the pause tells you more about the context than your read in step 1 did, so follow it.

Ask one multiple-choice question per point, all at once.

- Give every question two ways out besides answering, so the user never has to guess to get past it:
  - "I don't know": the user wants this point and does not have it yet. Without this option the only move left is a guess, and a point passed by a lucky guess gets recorded as understood.
  - "Not interested": the user did not come for this point. Whether they could answer it does not matter.
- Beside them, offer the correct answer and one to three wrong ones, most common mix-up first. A question needs at least four options to hold all of this; if the question tool takes fewer, ask in the message body.
- Vary the position of the correct option and never mark one as recommended. Do not hint at correctness in the option descriptions.

When the answers come back, say for each question whether it was right and why. For a wrong answer or "I don't know", explain again from a different angle (another example, a redrawn diagram) rather than repeating yourself, and check again with a different question. Repeating the same question lets a memorized answer pass.

For "Not interested", drop the point from this run without explaining it again or asking why. It is not a failed point and not a candidate for a later run. It does say your read of the context was off: if most of the points come back this way, stop and ask what the user came for instead of checking further. If the gist itself comes back this way, there is no page to write; end the run there.

Once the multiple-choice questions are settled, ask the user to write what the thing is in one or two sentences. That text becomes the basis of the page's gist. If it disagrees with the source, point out where and ask them to restate it.

Explain each point again about once. A point that still does not pass is left out of this run.

## 4. Settle the page plan

If the gist did not pass, say where understanding stopped and end without writing.

Otherwise, without writing any file yet, present:

- which points passed, which did not, which were dropped as not of interest, and the candidates for later runs
- the gist, based on the user's own sentences. Tidy the style, but do not rephrase beyond that
- a title, and a context folder only if the title collides with an existing page (check with `scraps get "<title>" --json`)
- the connections that passed, as links to existing pages with their direction
- tags chosen from `scraps tag list --json`

A page's subject is either a thing (a term, tool, or product) or the source itself (an article or talk: who said what, and where). A tool's official site yields a page about the tool. An article or talk yields a page titled with the source's own title, neither translated nor shortened.

Then ask only about the items where judgment could go either way. The recommended plan passes by default, so do not turn settled items into questions.

- For items that can coexist, such as tags and link targets, let the user pick several, put the recommended option first, and mark it as recommended.
- With more candidates than one question holds, list them all in the message body and ask only about those that need judgment.

Wait for the user to confirm or correct the plan.

## 5. Write the page

- Write only what passed the check.
- Order: if the title is an abbreviation, its expansion as a second-level heading; then the tag line, the gist, and bullets if any. Do not embed slides or videos.
- Record the source in the frontmatter, not in the body. A page about the source itself is an alias for it: give it the source's URL as `resource`, and add `sources` only for what was read when that differs, such as a translation. Any other page lists what it was written from in `sources`: the source's URL, or the path of the source's own page (`resource: "/<key>.md"`) when the wiki has one; step 7 finds those.
- The gist alone is the default. Add bullets only to state connections to existing pages. An explanation that carries no link is left out, even if the source has it.
- Link terms that have a page instead of explaining them. For terms without one, phrase the sentence so the term needs no explanation.
- Do not copy specification details, procedures, or exhaustive usage examples. That turns the page into a summary of the source.
- Write product classifications and comparisons only when a primary source verifies them.
- Reference pages with `[[Title]]`. Use `#[[Tag]]` only for existing cross-cutting categories.
- Point links from concrete to abstract. Do not list concrete examples from the abstract side.
- For a page about the source itself, name the author and the venue in the gist. Link people and organizations only when their page exists; otherwise leave them as plain text. Include dates and affiliations only when the source confirms them. Bullets carry connections, not the source's table of contents.

## 6. Link existing pages to the new one

1. Search again with the new title and read the `body` of each candidate to find literal mentions that are not yet links.
2. Drop mentions already inside a link, the new page itself, and candidates with no literal mention.
3. Present the remaining mentions with their context and your read of each, and let the user pick which to link, several at once. With more candidates than one question holds, list them all in the message body and ask only about those that need judgment.
4. Convert only the chosen mentions to `[[Title]]` or `[[Title|surface form]]`.

With no candidates, leave existing pages untouched. Backlinks are computed automatically, so do not add reverse links.

## 7. Verify

1. Reread the page. It should hold no explanation without a link and nothing that did not pass the check.
2. Confirm every tag is an existing tag or a new one the user explicitly agreed to.
3. Run `scraps lint --rule broken-link --rule source-has-page`. A `source-has-page` warning names the path to write in place of a source URL that has its own page.
4. Read the diff and confirm that only the new page and the approved links changed.
5. Report separately: the page created, the links updated, the points left out and the candidates for later runs, anything in the source that could not be read, and warnings unrelated to this change.
