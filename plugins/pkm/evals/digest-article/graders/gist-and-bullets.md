---
type: llm
focus: trace
---

Judge only the Markdown page the assistant wrote: the `content` argument of its Write tool call. Ignore everything else in the trace, including the assistant's closing report to the user, which may contain bullets of its own.

PASS if all of these hold: the page has a gist of one or two sentences that names who wrote or published the source and where; every bullet in the page contains at least one [[wiki link]]; the page does not reproduce the article's list of sections or its step-by-step details.

FAIL if any of them does not hold, or if no page was written.
