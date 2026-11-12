---
version: "0.1.1"
level: copilot
processes:
  design: pair
  implementation: copilot
  testing: copilot
  documentation: copilot
  review: hint
  deployment: none
convention:
  version: "0.1"
  level: 1
  adopted_at: <first commit>
  status: active
  tools:
    - "Claude Code:claude-opus-5-5"
    - "local:qwen3.8-27b"
  attribution_trailers: [Assisted-by]
  maintainers: [ikarys]
  review: "single maintainer reads every diff before committing to main; just check runs before each commit"
  security_response: "none"
  licence: Apache-2.0
  commands:
    test: "cargo test --workspace"
    lint: "cargo clippy --all-targets -- -D warnings"
    format: "cargo fmt --all --check"
    static_analysis: "cargo clippy --all-targets -- -D warnings"
---

# Vibecoding declaration

This project follows the [Vibecoding Convention](https://github.com/ikarys/vibecoding-convention)
version 0.1 at level 1. How each claim is checked is in [VERIFICATION.md](VERIFICATION.md).

## AI assistance

Almost all of the code and text is AI-written. The maintainer frames each change and decides;
Claude Code (Opus) writes the briefs, reviews and fixes; a local model (qwen3.8-27b) writes the
bulk code and drafts.

## Review

The single maintainer reads every diff and runs `just check` before committing to `main`. There
is no CI: the project is a ~300-line personal tool.

## Provenance and licence

This project is licensed under Apache-2.0. Parts of it are written by AI assistants and may not
be protected by copyright in some jurisdictions; the maintainer accepts this. To the maintainer's
knowledge, no verbatim reproduction of licence-incompatible code was included. Safeguards used:
human check of long generated blocks.

## Status

active. Security reports are handled on a best-effort basis, with no commitment.
