---
name: verify
description: Re-confirm an existing page of this Scraps wiki against today's primary sources together with the user, fix what has changed only as far as the user understands the change, and record in the page's `verified` frontmatter that it was checked, and the user's confirmation when they give it. When no page is named, takes stock of the wiki first and proposes the pages most worth checking. Use this whenever the user wants to know whether pages are still current, take stock or inventory of the wiki, update old pages to the latest, or re-check what a page says, even if they do not name a page. For a new page use learn. Broken or moved external links are out of scope.
argument-hint: "[page]"
---

# Verify

Check one existing page against how things stand today, with the user, and record that they confirmed it.

The wiki is a record of the user's understanding. A page was true to that understanding when it was written, but what it describes moves on: tools are renamed, options are removed, recommendations change. Rewriting the page from today's documentation would only swap the user's understanding for a fresh summary. So the changes are explained and checked the way `learn` checks a new page, and only what the user confirmed is written. The run can leave two `verified` entries, and they say different things. Yours, written as an agent, says the page was checked against its sources today; it is the date the next `verify` counts from. The user's, written `human:`, is their statement that the page still says what they understand; it is never added on the user's behalf. A run where every claim still holds, and only entries are added, has ended correctly.

Hold the dialogue and write in the user's language.

## Boundaries

- One run verifies one page. Name the other candidates for later runs.
- State a change as fact only after checking it in a primary source: official documentation, the repository, release notes, the changelog.
- Change only the page being verified. A change that belongs on another page, or something new that the page never covered, becomes a candidate for `learn`.
- Check a page about a source (an article, book, or talk) against that source as it stands today, not against the world. It records what the source said, so a claim is changed only when the source itself was revised: slides replaced, a post edited. That the world has moved on does not make it wrong.
- Check what the page says, not whether its links still resolve: note a broken or moved external link in the report and leave it. Do not widen the work into tidying the wiki.
- Add your own `verified` entry only when no claim is left changed, gone or unchecked on the page. Add the user's only for a page they confirmed as a whole.

## Asking the user

Follow `learn`'s "Asking the user" (`../learn/SKILL.md`).

## 1. Pick the page

If the user named a page, read it with `scraps get "<key>" --json` and go to step 2.

Otherwise take stock and propose:

1. Gather two pools, since neither alone finds the pages that drift:
   - pages about things that change quickly: tools, services, and their features. If the wiki types its pages, list them with `scraps type list --json` and `scraps type scraps "<type>" --json`.
   - pages not checked for a while: `scraps verified --by machine --older-than 180 --json` lists those whose newest check by an agent or process is more than that many days old, oldest first. A page never checked is not in that list; date it in the next step.
2. Date each page by when it was last checked against its sources. A page with a `verified` entry by an agent or process counts from the newest one; a `human:` entry says the user stood behind the page, not that anyone compared it with a source, so it does not count here. A page never checked counts from when its content last changed: a bulk edit of frontmatter or links touches many pages at once and makes them look fresh, so look past such commits in `git log --format='%as %s' -- <file>`.
3. Read `scraps get "<key>" --json body,type,verified` and drop pages with no claims to check, such as a stub with an empty body. Rank what remains, highest first:
   - pages about things that change quickly, oldest check first
   - pages the user is working near now (`git log --since="1 month ago" --name-only --format=`)
   - pages many others depend on (`scraps backlinks "<key>" --json`)
4. Present three to five candidates with why each is likely to have drifted, and let the user pick one.

## 2. Check the page against today

1. Read the page and what it was written from: its `sources` and `resource`. Open the current primary source for each.
2. Split the page into its claims: the gist, each bullet, each stated fact.
3. Sort each claim into one of:
   - holds: today's primary source still says this
   - changed: the thing still exists, but the claim no longer matches
   - gone: the thing was removed, deprecated, or replaced
   - unchecked: no primary source could confirm it either way
4. Note in passing anything important the page never covered. It is not part of this run.

If every claim holds, go to step 4.

## 3. Check the changes with the user

Explain each changed or gone claim: what the page says, what is true now, since when, and where you confirmed it. Anchor the explanation on the page itself and the pages it links to, which the user already knows. Then pause so the user can read it and ask questions before any check.

Then check comprehension of the changes only, following `learn`'s step 3 (`../learn/SKILL.md`) for the question shapes and for the "I don't know" and "Not interested" ways out. Claims that hold are not quizzed again.

- A change that passes is rewritten in the user's own words; ask them for a sentence when it touches the gist.
- A change marked "Not interested" means the user no longer keeps that claim: offer to remove it.
- A change that does not pass after one more explanation is left as it is, and the page is not confirmed in this run.

## 4. Confirm and settle the edit

Present, without editing yet:

- the claims that hold, the changes that passed with their new wording, the claims to remove, and those left unchecked
- the primary sources to add to `sources`, when one was used that the page does not list
- the candidates for later runs: other pages, and what the page never covered

Show the page as it would read after the edits, and ask the user to confirm it as a whole: whether it says what they understand today. When a claim mixes a part that holds with a part the user dropped, offer both trimming it and removing it. An unchecked or failed claim blocks the confirmation unless the user removes it or confirms it from their own knowledge.

## 5. Edit the page

1. Apply only the confirmed edits. Follow `learn`'s step 5 for how a page is written; keep the page's existing structure and wording elsewhere.
2. Add to the top of the frontmatter's `verified` list, each with the current time and its offset (`date -Iseconds`):
   - the user's entry, if they confirmed the page as a whole. Take the actor from project instructions; if they name none, ask the user once for a `human:<id>`.
   - below it, your own entry, if no claim is left changed, gone or unchecked. `by` is you as an agent, written `<producer>/<version>` (for example `claude-code/<model id>`).

   ```yaml
   verified:
     - by: human:<id>
       at: 2026-11-12T21:00:00+09:00
     - by: <producer>/<version>
       at: 2026-11-12T21:00:00+09:00
   ```

   Keep earlier entries; they are the history of confirmations.
3. If the page was not confirmed, keep the edits that passed and add no entry by the user. If a claim is still changed, gone or unchecked, add none of your own either: the page is not checked until it is.

## 6. Check

1. Confirm the entries with `scraps get "<key>" --json verified`: each one you added is listed, and the tier is `human-reviewed` when the user's is among them, `machine-confirmed` when only yours is.
2. Run `scraps lint --rule broken-link --rule source-has-page`.
3. Read the diff and confirm that only this page changed.
4. Report: what was confirmed, what changed and why, what was removed, what is still unchecked, and the candidates for later runs with the skill to use for each (`verify` or `learn`).
