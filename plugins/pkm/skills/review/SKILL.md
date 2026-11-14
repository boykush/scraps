---
name: review
description: Check again that the user still understands pages of this Scraps wiki whose understanding was confirmed long ago, and renew the confirmation of those that pass. Takes the pages whose `verified` entry is older than the wiki's threshold, asks the user to recall each before showing it, and appends a new `verified` entry on a pass; a page that does not pass is left untouched and stays due. Use this whenever the user wants to review, revisit, or be quizzed on what they wrote in the wiki, or asks what they may have forgotten. To understand something new, use learn.
argument-hint: "[key]"
---

# Review

Ask the user to recall what a page records, and renew its confirmation only when they still hold it.

`learn` writes a page once the user has shown they understand it, and records that moment as `verified`. Understanding fades while the page stays, so a wiki that is only written to drifts into a record of what its owner once knew. Review closes that gap: the page comes back, the user answers for themselves, and the date moves only when the answer holds.

The goal is the user's recall, not a tidy wiki. A run that renews nothing, because nothing was recalled, has ended correctly.

Hold the dialogue in the user's language.

## Boundaries

- Change nothing in a page except its `verified` entries. A page whose content looks wrong or out of date is reported, not fixed; rewriting it is a new `learn` run.
- Append a `verified` entry only for a page the user passed in this run. Do not renew a page because it is short, obvious, or because the user says they remember it.
- Do not show a page, quote it, or name its links before the user has answered for it. An answer given with the page in view measures reading.
- Do not write the user's answer for them, and do not let a wrong answer pass by quietly correcting it.
- Do not widen the work into tidying or auditing the wiki.

## Asking the user

Follow "Asking the user" in `learn` (`../learn/SKILL.md`): use the client's multiple-choice tool when it has one and the question fits it, otherwise ask in the message body. The user can always answer in their own words.

## 1. Pick the pages

1. Given a key, review that page, whether or not it is due.
2. Otherwise run `scraps lint --rule stale-verified`. Each warning names a page whose newest confirmation is older than the threshold, with its age in days. The threshold is the wiki's own policy, set in `[lint.stale_verified]` of `.scraps.toml`, and 180 days without it.
3. With no warning, say that nothing is due and end. Pages without any `verified` entry are never due: nobody confirmed them, so there is nothing to renew. That is `learn`'s ground.
4. With more than three due, take the three confirmed longest ago and say how many remain. A short run that gets finished beats a long one that gets abandoned.

## 2. Read the page, out of the user's sight

For each page, run `scraps get "<key>" --json body,verified,sources` and `scraps links "<key>" --json`, then read the pages it links to with `scraps get "<linked key>" --json` as far as the questions need.

From the page take the points to check, the same ones `learn` recorded: the gist saying what the thing is, and each connection to another page that a bullet states. A sentence that carries neither is not a point.

## 3. Ask for recall

Take the pages one at a time.

1. Name the page by its title only, and ask the user to say in one or two sentences what it is. Offer two ways out besides answering: "I don't remember", and "Skip this page" for one they do not want to review now.
2. Compare the answer with the page's gist.
   - The core matches: the gist passes, even when the wording differs or details are missing.
   - The core is wrong or missing, or the user does not remember: the gist does not pass. Go to step 4 for this page without asking about its connections.
3. For a page whose gist passed, ask one multiple-choice question per connection, all at once, built as `learn`'s "3. Check comprehension" builds them: apply the concept to a case, tell it apart from a nearby page, or say how it relates to the linked page. Give every question an "I don't remember" option, vary the position of the correct one, and mark none as recommended.

A page passes when its gist passed and at most one connection was missed. Connections are details of a thing the user still holds; missing most of them means the page no longer describes what they know.

## 4. Show the page

After the answers for a page are in, show what the page says and where the answers differed from it, for a pass as much as for a miss. Seeing the page right after trying to recall it is what makes the next recall easier.

- For a miss, explain the missed point once from the page and its linked pages. Do not ask again in this run: an answer given a minute after reading the page shows nothing about what will be there next month. The page stays due and comes back in the next run.
- When the user says the page is wrong, or that they now understand the thing differently, do not judge it and do not edit. Leave the confirmation as it is and name the page as a candidate for `learn`.
- A skipped page is left as it is, with nothing shown.

## 5. Renew the confirmation

For each page that passed, add one entry to the frontmatter `verified`, above the entries already there, and keep those:

```yaml
verified:
  - by: human:<id>
    at: <now>
  - by: human:<id>
    at: 2026-01-10T09:00:00Z
```

- `by` is the user, written as the page's existing `human:` entries write them. If the page has none, take the id from project instructions, else ask once.
- `at` is the current time: `date -u +%Y-%m-%dT%H:%M:%SZ`.
- A `verified` written as one bare mapping becomes a list of two.

Change nothing else in the file.

## 6. Verify and report

1. For each renewed page, run `scraps get "<key>" --json verified` and confirm the newest entry is the one you wrote. An entry of the wrong shape is dropped without a warning, so only this shows it was read.
2. Run `scraps lint --rule stale-verified` and confirm the renewed pages are gone from it and the others are still there.
3. Read the diff and confirm that only `verified` lines changed, and only in pages that passed.
4. Report separately: the pages renewed, the pages that did not pass and which point each missed, the pages skipped, the candidates for `learn`, and how many pages remain due.
