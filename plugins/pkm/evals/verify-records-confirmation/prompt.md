---
description: When every claim on the page still holds and the user has confirmed it as a whole, verify adds one verified entry by the user's actor, which makes the page human-reviewed, and changes nothing else.
tags: [verify]
max_turns: 40
timeout_seconds: 600
allowed_tools: [Read, Glob, Grep, Skill, AskUserQuestion]
---

Check my AI agent page against its source. I re-read it this morning: if every claim still holds, take this as my confirmation of the page as a whole and record it. My actor is human:alice.
