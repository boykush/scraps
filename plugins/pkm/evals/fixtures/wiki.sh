#!/usr/bin/env bash
# Builds the fixture wiki in the current directory. Sourced by each case's fixture.sh,
# so every case starts from the same pages and tags.
set -euo pipefail

scraps init

cat > "LLM.md" <<'PAGE'
## Large Language Model

#[[AI]]

A neural network trained on large amounts of text that generates language by predicting the next token.
PAGE

cat > "AI agent.md" <<'PAGE'
#[[AI]]

A system in which an [[LLM]] decides its own steps and tool calls in a loop until a task is done.

- Acts through [[Tool use]]
PAGE

cat > "Tool use.md" <<'PAGE'
#[[AI]]

The ability of an [[LLM]] to call external functions that are described to it, and to read their results.
PAGE

cat > "Anthropic.md" <<'PAGE'
#[[AI]] #[[Company]]

The AI company that develops the Claude models.
PAGE

cat > "Prompt engineering.md" <<'PAGE'
#[[AI]]

The practice of designing the instructions given to an [[LLM]] so that it produces the intended output.
PAGE

cat > "RAG.md" <<'PAGE'
## Retrieval-Augmented Generation

#[[AI]]

Supplying an [[LLM]] with documents retrieved at query time so that its answer is grounded in them.
PAGE
