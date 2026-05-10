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
❯ scraps links "Getting Started" --json
❯ scraps backlinks "Configuration" --json
❯ scraps tag list --json
❯ scraps todo --status all --json
```

`scraps get --json` defaults to `title`, `ctx`, and `body`. It can project
specific fields (`title`, `ctx`, `body`, `headings`, `code_blocks`) so an
agent can avoid loading full bodies when it only needs structure or examples.
`scraps links --json` returns outbound `link` and `embed` references with
optional heading targets; `backlinks` stays a scrap-level inbound lookup.

The full command map is in [[Reference/CLI Overview]]. Each command's `--help`
documents flags and JSON shape.

### Bundled AI skills

For Claude Code users, the official **scraps plugin** packages
[Karpathy](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)-style
*Ingest / Query / Lint* workflows around the CLI:

<https://github.com/boykush/scraps/tree/main/plugins/scraps>

| Skill / Agent | Role |
|---|---|
| `/ingest` | Add a new scrap from a prompt, URL, or markdown; update cross-links |
| `/query` | Answer a question against the wiki with `[[Title]]` citations |
| `lint-rule-handler` agent | Purpose-driven wiki health checks, one or a few rules at a time |

Install instructions live in the plugin README so that marketplace browsers
have everything in one place.

## MCP (for MCP-compatible clients)

Scraps ships an MCP server for clients that prefer the Model Context
Protocol. The server is bundled as a plugin so installation and tool
specifications stay together:

<https://github.com/boykush/scraps/tree/main/plugins/mcp-server>

To wire the MCP server into Claude Code manually without the plugin:

```bash
claude mcp add scraps -- scraps -C ~/path/to/your/wiki mcp serve
```

Replace `~/path/to/your/wiki` with the directory containing `.scraps.toml`.

For most read-shaped agent workflows, the CLI + JSON path above is simpler:
no long-running process, no MCP client implementation required, works with
any shell-capable agent. MCP is the right choice when your agent already
expects MCP tools as its integration surface.
