#[[Integration]] #[[Emit/CLI JSON]]

Scraps integrates with AI assistants in two ways. **CLI + JSON is the primary
path** because shell commands plus structured output are the lowest-friction
contract any agent can use. MCP is supported for clients that expect Model
Context Protocol tools.

## CLI + JSON (recommended)

Any assistant with shell access can query Scraps without a long-running
server.

```bash
❯ scraps search "rust cli" --logic and --json
❯ scraps get "Getting Started" --json
❯ scraps get "Getting Started" --heading "Install" --json body
❯ scraps get "Getting Started" --json code_blocks
❯ scraps get "Getting Started" --json images
❯ scraps links "Getting Started" --json
❯ scraps backlinks "Configuration" --json
❯ scraps tag list --json
❯ scraps todo --status all --json
```

`scraps get --json` defaults to `title`, `ctx`, and `body`. It can project
specific fields (`title`, `ctx`, `body`, `headings`, `code_blocks`, `images`) so an
agent can avoid loading full bodies when it only needs structure or examples.
`scraps links --json` returns outbound `link` and `embed` references with
optional heading targets; `backlinks` stays a scrap-level inbound lookup.

The full command map is in [[Reference/CLI Overview]]. Each command's `--help`
documents flags and JSON shape.

### Bundled AI skills and agents

For Claude Code and Codex users, the official **scraps plugin** packages
[Karpathy](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)-style
*Ingest / Query / Lint* workflows around the CLI. The Claude Code agents add
purpose-driven lint handling and a default Scraps LLM Wiki schema grounded in
the official docs:

<https://github.com/boykush/scraps/tree/main/plugins/scraps>

| Skill / Agent | Role |
|---|---|
| `/ingest` | Add a new scrap from a prompt, URL, or markdown; update cross-links |
| `/query` | Answer a question against the wiki with `[[Title]]` citations |
| `lint-rule-handler` agent | Purpose-driven wiki health checks, one or a few rules at a time |
| `scraps-llm-wiki-schema` agent | Explain Scraps tool usage from the official docs and map LLM Wiki practice to `ingest`, `query`, and `lint-rule-handler` |

Install instructions live in the plugin README so that marketplace browsers
have everything in one place.

## MCP (for MCP-compatible clients)

Scraps ships an MCP server for clients that prefer the Model Context
Protocol. It serves over stdio by default, or over Streamable HTTP with
`--http` so a single process can back every repository on your machine:

```bash
❯ scraps -C ~/path/to/your/wiki mcp serve --http
```

This listens on `127.0.0.1:1113` and serves MCP at
`http://127.0.0.1:1113/mcp`. Every repository points its client at that one
URL, so the wiki path is configured once — on the server — instead of in
each repository. One process serves one wiki; run a second process on
another port to serve another.

### Behind a proxy or tunnel

The HTTP transport accepts only loopback `Host` headers, which is what keeps
a browser on a visited page from reaching a server bound to your machine. A
reverse proxy or tunnel forwarding under its own hostname is refused with
`403 Forbidden: Host header is not allowed`, so name that hostname when
starting the server:

```bash
❯ scraps -C ~/path/to/your/wiki mcp serve --http 0.0.0.0:1113 \
    --allowed-host wiki.example.com
```

Repeat `--allowed-host` for each hostname. Loopback stays allowed either
way, so a port-forward or a local `curl` against the same process keeps
working. Naming the hostname here is preferable to having the proxy rewrite
`Host`: the allowlist stays as narrow as the deployment actually needs, and
it stays next to the server it applies to rather than in proxy
configuration.

The server is bundled as a plugin so installation and tool specifications
stay together:

<https://github.com/boykush/scraps/tree/main/plugins/mcp-server>

To register the running server with Claude Code manually, without the
plugin:

```bash
❯ claude mcp add --transport http scraps http://127.0.0.1:1113/mcp
```

For a single repository, stdio needs nothing running — the client spawns
Scraps itself:

```bash
❯ claude mcp add scraps -- scraps -C ~/path/to/your/wiki mcp serve
```

Replace `~/path/to/your/wiki` with the directory containing `.scraps.toml`.

For most read-shaped agent workflows, the CLI + JSON path above is simpler:
nothing to keep running, no MCP client implementation required, works with
any shell-capable agent. MCP is the right choice when your agent already
expects MCP tools as its integration surface, and `--http` is the right
transport when several repositories share one wiki.
