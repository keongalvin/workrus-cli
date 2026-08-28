---
feature: rust-linear-cli
requirement_doc: null
created: 2026-08-28
---
# Rust Linear CLI

Build `workrus`, a small, low-dependency Rust CLI inspired by `schpet/linear-cli` for issue-focused Linear workflows.

## Design: Level 1 — Capabilities

1. Authenticate to Linear using an API key supplied through the environment.
2. Configure a repository's default Linear team.
3. List, query, view, create, and update Linear issues with human-readable or JSON output.
4. Start work on an issue by creating a Git branch and moving the issue into a started workflow state.
5. Infer the current issue from a Linear identifier embedded in the active Git branch.

## Approved v0.1 Command Surface

- `workrus config`
- `workrus team list`
- `workrus issue mine` (aliases: `list`, `l`)
- `workrus issue query`
- `workrus issue view [ID]`
- `workrus issue create`
- `workrus issue update [ID]`
- `workrus issue start [ID]`
- `workrus issue id|title|url [ID]`

## Implementation Plan

1. Bootstrap the Cargo package, MIT license, formatting, linting, CI, and release configuration.
2. Keep a documented dependency budget and prefer synchronous, focused crates; `anyhow` is permitted.
3. Implement CLI parsing, typed boundary validation, configuration precedence, GraphQL transport, pagination, and actionable errors.
4. Implement team discovery, issue identifier parsing, Git branch inference, read commands, write commands, and start-work integration as vertical slices.
5. Provide stable JSON output and compact terminal formatting without a heavy UI framework.
6. Add unit, mock-HTTP, and CLI integration coverage without requiring live Linear credentials.
7. Document installation, configuration, upstream inspiration, command compatibility, and intentional omissions.

## Decisions Log

| Date | Decision | Reasoning | Alternatives Considered |
|------|----------|-----------|------------------------|
| 2026-08-28 | Implement a fresh Rust CLI rather than a line-for-line port. | The upstream Deno CLI has a broad command surface; a selective implementation better serves the small, low-dependency goal. | Full feature parity and mechanical translation were rejected as excessive for v0.1. |
| 2026-08-28 | Name both the crate and binary `workrus`. | Matches the repository and user approval. | Shipping as `linear` was not selected. |
| 2026-08-28 | License the project under MIT, copyright Alvin Yip. | Explicit user direction. | ISC and dual MIT/Apache-2.0 were not selected. |
| 2026-08-28 | Support Git only and omit `jj`. | Explicit user direction to avoid `jj` support. | Upstream Git-plus-`jj` parity was rejected. |
| 2026-08-28 | Optimize for a small direct dependency set; `anyhow` is allowed. | Low dependency count remains a project goal, while `anyhow` can keep application-level error handling concise. | A strict no-error-helper policy was relaxed. |
| 2026-08-28 | Start with `LINEAR_API_KEY` authentication rather than multi-workspace keyring management. | Keeps v0.1 small and avoids platform-specific keyring complexity while keeping secrets out of source and project config. | OS keyrings, plaintext credential files, and multi-workspace auth are deferred. |
| 2026-08-28 | Preserve familiar upstream issue command names where they fit the approved scope, without promising full compatibility. | Reduces learning cost while allowing a smaller design. | A wholly new CLI vocabulary and full compatibility were rejected. |

## Open Questions

- None blocking implementation. Exact dependency choices may change if validation reveals a materially smaller or safer option.

## Constraints

- The crate and binary are named `workrus`.
- The project uses the MIT license with Alvin Yip as copyright holder.
- Git is the only VCS integration; do not add `jj` support.
- Keep runtime dependencies deliberately small; every direct dependency must earn its place.
- Never persist or log Linear API keys.
- Do not expand v0.1 into upstream's projects, cycles, milestones, initiatives, documents, comments, attachments, keyrings, pagers, or shell-completion surface.
- Maintain stable machine-readable JSON output for supported commands.

## Key Files

- `README.md` — current placeholder; replace with project documentation during implementation.
- `.lattice/context/rust-linear-cli.md` — approved scope, constraints, and decisions.
