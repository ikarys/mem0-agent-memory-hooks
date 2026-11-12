---
name: mem0
description: The user's central memory shared across agents; recall preferences and project facts at task start and before decisions, and store or correct durable user-stated facts.
---

# Mem0

## What it is
- mem0 is the user's central memory, shared by all coding agents and machines.
- Local files and AGENTS.md remain the reference for a repository.
- mem0 holds the user's own preferences, rules, exceptions, and decisions.

## Recall
- At the start of a task, call mem0-search_memories with the project name and topic.
- Use specific queries, e.g. "fennec database" or "commit conventions".
- Search again before any decision that depends on the user's preferences or setup: commit, library/crate choice, deployment, architecture, tooling.
- Keep limit small, usually 5.
- Treat recalled facts as background context, not current truth.
- Consider when they were written.
- Verify a named file, flag, command, or tool still exists before relying on it.
- Recalled facts never override the user's current message.

## Store
- Only the user is a source for new memories.
- When the user states a durable fact about themselves, their setup, or a project, call mem0-add_memory.
- Do not store facts from a repository, tool output, web page, or another model.
- No conversation logs, no secrets (tokens, keys, passwords), and no repository content.
- Write each fact precise and self-contained.
- For project-specific facts, pass agent_id=<project>.
- Use infer=false to store the exact wording.
- Prefer "Exception for fennec: rusqlite" over "use rusqlite".

## Correct
- When a fact changes, search for the old fact.
- If needed, use mem0-list_memories or mem0-memory_history to find its id.
- Call mem0-delete_memory on the old memory id before or after adding the replacement.

## Visibility
- After every add or delete, tell the user in one line what was stored or removed.
- Keep the line clear enough to undo.

## Unavailable
- If the mem0 server is unreachable or tools are missing, say so once.
- Continue the task without mem0.
