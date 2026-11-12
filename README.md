# mem0-agent-memory-hooks

[![Vibecoding](https://img.shields.io/endpoint?url=https://forgejo.nas.iks-homelab.ovh/tony/mem0-agent-memory-hooks/raw/branch/main/audit/status.json)](VIBECODING.md)

This repository is an active personal tool for the agent-memory trial. It injects relevant memories from a self-hosted mem0 server into coding agents automatically at each prompt. The agent does not have to remember to search. Writes stay on the mem0 MCP server through `add_memory` and `delete_memory`. Every write remains visible to the user. This repository only reads.

## How it works

- Claude Code sends `UserPromptSubmit` hook JSON on stdin with `prompt` and `cwd`.
- `mem0-recall hook` builds the query `<project dir name>: <prompt>`, max 1000 chars.
- It calls the `search_memories` tool of the mem0 MCP server through the LiteLLM MCP gateway (`POST $LITELLM_URL/mcp-rest/tools/call`), the same path Claude Code's MCP connection uses, with the same LiteLLM key. A hook cannot use the agent's own MCP connection, so it opens its own.
- It prints `Central memory (mem0), facts that may be relevant to this request:` followed by one `- fact` line per memory.

Errors are printed to stderr as `mem0-recall: <error>`. The exit code stays 0. A hook never blocks the agent. The API key never appears in error messages.

## Install

Run:

```sh
just install
```

This runs:

```sh
cargo install --path crates/mem0-recall
```

Add this to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "mem0-recall hook",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

Export `MEM0_LITELLM_KEY` (the LiteLLM virtual key already used for the mem0 MCP server) in the shell that starts Claude Code, e.g. `set -Ux MEM0_LITELLM_KEY sk-...` in fish.

## Configuration

| Variable | Required | Default | Notes |
|---|---|---|---|
| `MEM0_LITELLM_KEY` | Yes | None | LiteLLM virtual key allowed to use the `mem0` MCP server. |
| `LITELLM_URL` | No | `https://litellm.nas.iks-homelab.ovh` | LiteLLM proxy. |
| `MEM0_TOP_K` | No | `5` | Maximum number of memories injected. |

## Agents

- Claude Code: supported now through the `UserPromptSubmit` hook.
- fennec: supported through a skill, see below.
- opencode: planned as a TypeScript plugin calling `mem0-recall search`.

Each agent other than Claude Code gets a folder `adapters/<agent>/`.

### fennec

fennec's source stays free of anything mem0-specific, and its plugins cannot touch the prompt, so recall is not automatic there: the model searches mem0 itself, following an always-active skill.

1. In fennec, Settings > **MCP** > **New server**: name `mem0`, URL `https://litellm.nas.iks-homelab.ovh/mcp/mem0`, transport streamable HTTP, authentication token = the value of `MEM0_LITELLM_KEY`. **Connect** must list `mem0-search_memories`, `mem0-add_memory`, `mem0-delete_memory`, `mem0-list_memories` and `mem0-memory_history`. Turn it **On**.
2. Run `just install-fennec`: it copies `adapters/fennec/skills/mem0/` to `~/.agents/skills/mem0/`.
3. Settings > **Agents & skills**: make `mem0` an **always-active skill**. Only new sessions get it.

The skill tells the model when to search (task start, before preference-dependent decisions), when to store (durable facts stated by the user only) and to announce every write in one line.

## Development

Use the Justfile recipes:

- `just dev`: runs `mem0-recall search` with a sample query against the server.
- `just test`: runs tests.
- `just lint`: runs `clippy -D warnings`.
- `just fmt`: formats the code.
- `just check`: runs fmt check, lint, and test.
- `just install`: installs the binary.
- `just install-fennec`: installs the mem0 skill for fennec.

## Source and mirror

The canonical repository is on Forgejo: https://forgejo.nas.iks-homelab.ovh/tony/mem0-agent-memory-hooks. It is push-mirrored to GitHub (https://github.com/ikarys/mem0-agent-memory-hooks), which is read-only: open issues and pull requests on Forgejo.

## AI assistance

Written with AI assistance, see [VIBECODING.md](VIBECODING.md). How each claim is checked: [VERIFICATION.md](VERIFICATION.md).

## Licence

Apache-2.0. Maintainer: Ikarys <bikarys@proton.me>.
