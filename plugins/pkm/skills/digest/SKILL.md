---
name: digest
description: Write one page about an article, blog post, news item, release note, or talk (slides) from its URL into this Scraps wiki, without dialogue - a short summary of what the source says, linked to the pages the wiki already has. Use this when the user hands over the URL of something they read or want to keep up with and asks to put it in the wiki, record it, or summarize and keep it; write it through without asking for confirmation. For anything the user wants to understand, or for a page whose subject is a term, tool, or product, use learn instead.
argument-hint: "[url]"
---

# Digest

Take the URL of an article or talk and write one page about that source, start to finish, without dialogue.

Digest is for keeping up with incoming material. The page records what the source says and where it touches what the wiki already holds, so the user can take it in quickly. It does not claim the user understood it; that is `learn`'s job. The subject is therefore always the source itself (who said what, and where), never a term or a tool. A page named after a concept stands for something the user understood, and only `learn` writes those.

Do not ask questions along the way. Where judgment could go either way, decide by the rules here and record the decision in the report. If the rules do not decide it, leave that part unwritten and report it. The user can read a diff and fix it, but an understanding that was silently added goes unnoticed.

## Scope

Digest handles URLs where the page's subject is the source itself: technical blog posts, news articles, talks, release notes, and explanatory pages of official documentation.

When the URL points at the official site of a tool or product, the subject is the thing rather than the source (the Scalafix site becomes a page named Scalafix). Hand those to `learn`, along with any URL you cannot classify.

## Boundaries

- Do not change existing pages at all. Connect only through links from the new page; backlinks are computed automatically. Turning mentions in existing pages into links needs a judgment of context (some languages have no word boundaries), which `learn` makes together with the user in its step 6.
- One page per source. Given several URLs, write them one at a time and never merge them into one page.
- Do not also create pages for the concepts the source mentions.
- Do not create pages for people, organizations, or publications. Without an existing page, write them as plain text.
- Do not create tags. If no existing tag fits, shorten the tag line.
- Do not widen the work into tidying or auditing the wiki.

## 1. Read the source and the ground

1. Fetch the source.
2. Check how much you got. Slide hosts such as Speaker Deck often yield only the title and abstract. Write from what you could read, note in the report what you could not, and do not fill the gap by guessing.
3. Identify the central terms and claims.
4. Search for related pages with varied wording using `scraps search "<query>" --json`.
5. Check the existing tags with `scraps tag list --json`.
6. Read the 3–8 most related pages with `scraps get "<title>" [--ctx "<ctx>"] --json`.
7. Split the central terms into those that have a page and those that do not. A term has a page only when that thing is the page's subject. A page about a source (an article, a talk, one entry of a report or catalog) is not the page for the term in its title, even when the title is exactly the term; read the body to tell, and link such a page only when you mean that source. Link the former and leave the latter out of this page.
8. Look up the source's title with `scraps get "<source title>" --json` to see whether a page of that name exists.

## 2. Write the page

Follow the writing rules in `learn`'s "5. Write the page" (`../learn/SKILL.md`), read as follows.

- "Write only what passed the check" reads as: write only what the source confirms.
- Do not use the abbreviation heading. The title is the source's own title, neither translated nor shortened.
- "Do not copy specification details, procedures, or exhaustive usage examples" applies as is. The reason here is not that it would summarize the source (the gist of a digest page is a summary) but that it is explanation without a connection.

The page has this form:

```
#[[Tag]] #[[Tag]]

<Gist: who says what, and where. One or two sentences.>

- <A bullet that carries a connection to an existing page. Zero to three.>

<URL>
```

Place the page without a context folder by default, and add one only when step 1 found a page with the same title. Project instructions may add local conventions for source pages, such as a category link or the context folder to use; follow them.

## 3. Verify and report

1. Reread the page and confirm no explanation without a link remains.
2. Confirm every tag is an existing tag.
3. Run `scraps lint --rule broken-link`.
4. Read the diff and confirm nothing changed except the one new file.
5. Report separately: the page created with its tags and links, the decisions you made by rule, what could not be fetched from the source, the reason for a context folder if one was used, and warnings unrelated to this change.
