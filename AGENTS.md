<!-- AI_COCKPIT_ADAPTER_BEGIN provider=codex adapterVersion=1 repositoryId=sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b -->

This repository is attached to AI Cockpit.

Canonical interface: .ai/agent-interface.json

Use AI Cockpit as the repository-governance interface.
Prefer MCP when available; CLI remains the fallback.

Do not infer AI Cockpit state from this file.
Query the Runtime for current governance state.

<!-- AI_COCKPIT_ADAPTER_END -->

## AI Cockpit repository workflow

Read `.ai/README.md` before changing this repository. Use the installed shared
Runtime with an explicit `--repo /path/to/ai-cockpit` on every repository-bound
command. Query `inspect`, `status`, and `doctor` before acting; use the Work Item
lifecycle `start → preflight → checkpoint → verify → finish → archive → close`
for authorized changes. Do not infer state from this file, edit global Agent or
MCP configuration, or claim governance outcomes without current Runtime evidence.

## Outcome and release acceptance boundaries

When an Agent needs a result for a person, use the human Outcome handoff
(`work-item outcome` or the repository-bound MCP `work_item_outcome` tool).
`work_item_get` is a machine-oriented record lookup and is not a substitute for
the visible handoff. Preserve Contract acceptance criteria in their original
language; presentation localization must not alter governance facts or create
human decisions.

Release adopter acceptance must retain its isolation receipt and manifests.
HOME and XDG_CONFIG_HOME are forbidden-write roots; TMPDIR and CARGO_HOME are
explicitly isolated, classified runtime-write roots. A passing receipt must bind
the source repository, repository identity, root manifests, metadata, and
digests, and must prove the temporary run root was cleaned up.

## Work Item change discipline

Use one active Work Item, one dedicated branch, and one pull request for a
change. Start the branch from the latest remote default branch and keep the
Contract scope, out-of-scope boundary, evidence, and verification commands
current. If an in-scope defect is discovered, amend and verify the current
Contract before opening another Work Item; do not hide it in a later task.

Merge only the reviewed PR after its hosted checks pass. Do not use local-main
as a substitute for pre-merge review. After merge, synchronize the default
branch, prove the Work Item is closed, and remove only the exact merged branch
and worktree after cleanup is verified.
