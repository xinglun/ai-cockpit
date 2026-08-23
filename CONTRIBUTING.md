# Contributing

AI Cockpit is a Rust Runtime and repository-governance project. Contributions
should preserve the explicit repository boundary, fail-closed evidence rules,
and the reader-facing documentation route.

Before changing code, tests, documentation, CI, or governance files:

1. Read `.ai/README.md` and `.ai/glossary.md`.
2. Query the installed Runtime with an explicit `--repo` path.
3. Use one Work Item, one dedicated branch/worktree, and one reviewed PR.
4. Confirm the Contract's scope, out-of-scope boundary, authority, acceptance,
   evidence, and verification commands before implementation.
5. Stop for human review when preflight is `not_ready` or
   `needs_human_confirmation`; a successful command is not an authorization.

Keep the shared `ai-cockpit` binary external to governed repositories. Do not
copy Runtime code, use a source checkout as a release substitute, or modify
user-global Agent/MCP configuration. Repository-bound commands must carry
`--repo`, and generated status, receipt, archive, and decision records must be
written by the Runtime rather than hand-edited.

Add or update deterministic tests for behavior changes. Preserve unknowns and
historical evidence; do not weaken a gate to make a check pass. Use the
human-facing Outcome handoff with its 🟢/🟡/🔴 marker before finish, archive,
merge, close, or release progression.

After a PR is merged, verify the merged head, archive and decision receipts,
default-branch synchronization, clean relevant worktrees, and exact branch
removal. Release and upgrade claims must use immutable published artifacts and
retain their acceptance manifests.

For security reports, use [SECURITY.md](SECURITY.md). For the user-facing
project route, start with [README.md](README.md).
