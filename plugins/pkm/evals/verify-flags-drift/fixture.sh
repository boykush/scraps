#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../fixtures/wiki.sh"

# The last bullet contradicts the page's own source, which recommends the simplest
# solution first and treats workflows and agents as two kinds of agentic system.
cat > "AI agent.md" <<'PAGE'
---
sources:
  - https://www.anthropic.com/engineering/building-effective-agents
---
#[[AI]]

A system in which an [[LLM]] decides its own steps and tool calls in a loop until a task is done.

- Acts through [[Tool use]]
- Anthropic recommends building every LLM application as an autonomous agent on a framework from the start, since workflows with predefined code paths are a legacy pattern.
PAGE
