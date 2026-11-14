---
type: llm
focus: last_message
---

PASS if the reply says every claim on the page still matches the source, that a confirmation by human:alice was recorded (it may also say the agent's own check was recorded), and that the page is now human-reviewed.

FAIL if it reports a claim as changed or gone, rewrites the page, or does not say the page is human-reviewed.
