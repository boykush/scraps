#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../fixtures/wiki.sh"

# Every claim here still matches the source, so the run has nothing to quiz.
cat > "AI agent.md" <<'PAGE'
---
sources:
  - https://www.anthropic.com/engineering/building-effective-agents
---
#[[AI]]

A system in which an [[LLM]] decides its own steps and tool calls in a loop, based on feedback from its environment.

- Acts through [[Tool use]]
PAGE
