# How it is built and verified

This is a ~300-line tool, written with AI assistance (see [VIBECODING.md](VIBECODING.md)); the checks below are proportionate to that size.

## Claims

| Claim | How it is checked |
|---|---|
| Pure logic: config parsing, MCP response parsing, query building, formatting | Unit tested with `cargo test` in `crates/mem0-recall/src/lib.rs`. |
| No clippy warnings | `just lint`. |
| Formatting | `cargo fmt --check`. |
| A hook never blocks the agent: every error path exits 0 | Checked by hand on 2026-11-12: missing key, wrong key (HTTP 401), unreachable server (3 s timeout). No automated test. |
| The API key is redacted from error messages | Code in `describe_error`. No automated test. |
| Read-only: the binary only calls the `search_memories` tool | Check by reading `lib.rs`: one HTTP call, tool name hard-coded. |

## Not checked

- No CI. A single maintainer runs `just check` before committing.
- No blind test yet. Adapters for opencode and fennec do not exist yet.
- No coverage threshold. Vibecoding Convention level 1.
