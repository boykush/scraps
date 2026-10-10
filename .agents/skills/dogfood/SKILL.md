---
name: dogfood
description: Improve Scraps' MCP server and agent skills from how they were actually used. Reads the traces the owner's own deployment leaves in Jaeger, turns one finding into a change, and opens the PR without stopping to ask in between. Owner only - the traces sit behind the owner's login. Use this whenever the owner asks to dogfood, to analyse how the MCP server or a skill is being called, or to find the next MCP improvement from real usage (e.g. "/dogfood", "MCP の呼ばれ方を分析して改善して", "先週の trace から次の改善を").
---

# Dogfood

The repository owner runs Scraps as a remote MCP server over their own wiki and uses it every day. Each request leaves a span. This skill reads those spans, finds where the tools and the way agents are told to use them fall short, and ships one improvement.

Only the owner can run it. The `jaeger` MCP server authenticates as them, and no one else gets past its login. This skill is also the only place in the repository that calls `mcp__jaeger__*`: outside it, leave those tools alone.

If the `jaeger` tools are missing or refuse the call, stop and say so. The package's README (`apm_modules/boykush/ai-plugins/plugins/jaeger-remote-mcp/README.md`) says how to sign in and when the cluster is down. Don't route around the login.

## 1. Read the traces

- Find the service with the services tool rather than assuming its name: it is the one whose spans come from the `scraps` instrumentation scope.
- Take the whole retention window unless the owner names another. A week of personal use is a small sample, so narrowing it further leaves little to read.
- `src/mcp/traced.rs` owns what a span is named and which attributes it carries. Read it before interpreting them, since it changes as the telemetry improves.
- A trace can hold other services' spans for the same request, such as a gateway in front. Count only the spans whose resource is the Scraps service, or each call is counted twice.

## 2. Find what the usage says

Set the calls beside what the server offers: `src/mcp/server.rs` for the tools, their descriptions and the server instructions, `src/mcp/tools/` for each request schema.

- **Tools never called.** A tool nobody reaches for is either unneeded for this kind of use or undiscoverable from where agents stand when they would want it.
- **Arguments that fight the schema.** A default passed explicitly every time, an optional parameter never used, a value retried in another form.
- **Sequences.** What follows what, and where a sequence ends early. A search with no read after it is a question the response could not answer.
- **Errors**, and whether the message told the caller what to do next.
- **Latency** that every tool shares, as opposed to one tool's own.

Set aside the calls that were not use: connectivity checks, and the calls this skill's own earlier runs caused. They are recognisable by their arguments and by arriving in a burst.

When the traces cannot answer a question that matters, that gap is a finding in its own right. A span that records the missing fact is a legitimate change to ship, and often the one to ship first.

## 3. Pick one change

Rank the findings by how much evidence stands behind each and how small the change is. Take the top one. Before writing code, state:

- the evidence, as counts over a dated window
- the change
- what the traces should show afterwards if it worked

The last one is what makes the next run able to judge this one. A change with no observable expectation is a guess.

Where the change lands depends on what was wrong:

| What was wrong | Where |
| --- | --- |
| A tool's behaviour, schema or response | `src/mcp/tools/`, `src/mcp/json/` |
| How agents are told to use the tools | Descriptions and instructions in `src/mcp/server.rs` |
| What the traces could not show | `src/mcp/traced.rs` |
| How a skill drives Scraps | `plugins/*/skills/` |

A changed tool also changes `plugins/mcp-server/README.md`, and a changed plugin skill needs its version bumped and its evals run (CONTRIBUTING.md).

## 4. Ship it

Follow the workflow in AGENTS.md: a branch from `origin/main`, TDD, Conventional Commits, one PR. Go from the finding to the open PR without asking for confirmation in between. The PR is where the owner reviews; don't merge it.

The PR body carries the evidence, the change and the expectation from step 3.

**Keep what the owner searched for out of the repository.** Span arguments are the owner's own queries, and this repository is public. Commits, PR text, tests and fixtures get counts and shapes ("26 of 28 searches passed the default explicitly"), never the query strings or scrap titles. Quoting them to the owner in the session is fine.

## 5. Close the loop

Report to the owner: the findings ranked, the one shipped with its PR, and the ones left for next time.

The next run starts by checking the previous one. Find the earlier dogfood PRs (`gh pr list --state merged --search "dogfood in:body"`), see whether each has reached the deployed version (the instrumentation scope's version on the spans), and compare the traces since then against the expectation it stated. An expectation that did not hold is the first finding of the new run.
