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

If `preflight` returns `not_ready` or `needs_human_confirmation`, stop and show
the Preflight Review to the human; an advisory zero exit status is not permission
to implement. Treat `.ai/README.md`, `.ai/glossary.md`, this file, and the
repository's current machine-readable governance records as the default
instruction read set. Historical `docs/archive/**` and reference material do
not grant current authority unless explicitly included by the human or Contract.

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

Use one active Work Item per branch, worktree, and repository context, with one
dedicated branch and one pull request for that Work Item. Compatible,
independent Work Items may run in parallel when their scopes and repository
contexts are isolated and the Runtime declares them compatible. Start each
branch from the latest remote default branch and keep the Contract scope,
out-of-scope boundary, evidence, and verification commands current. If an
in-scope defect is discovered, amend and verify the current Contract before
opening another Work Item; do not hide it in a later task.

The canonical delivery order is latest remote default base → dedicated
branch/worktree → implement → finish/archive → push → reviewed PR → merge →
close → synchronize and clean. Never merge a feature branch into local `main`
before PR review, delete its branch before merge, or let a provider auto-delete
it to bypass finalization. If a remote step fails, preserve the retry checkout
and identity until recovery is complete. A repository is `ready_on_base` only
after the reviewed merge, synchronized default branch, and exact cleanup have
been verified; a detached or otherwise unbound worktree is not ready for the
next Work Item.

Merge only the reviewed PR after its hosted checks pass. Do not use local-main
as a substitute for pre-merge review. After merge, synchronize the default
branch, prove the Work Item is closed, and remove only the exact merged branch
and worktree after cleanup is verified.

## Agent operating boundaries

Before editing code, tests, documentation, CI, build files, or governance
files, read `.ai/README.md` and `.ai/glossary.md`, then query `inspect`,
`status`, and `doctor` with the explicit repository path. Establish or identify
one active Contract with a human-owned intent, scope, out-of-scope boundary,
authority, acceptance criteria, required evidence, base revision, and declared
verification commands. Do not edit outside that scope, remove tests or
evidence records without recording the reason, or hand-edit generated status,
receipt, or archive files.

Each Work Item starts from the latest commit on the repository's discovered
remote default branch. Record the remote, default branch, and base revision in
the Contract. Installation and upgrade acceptance use an immutable published
Release tag and downloaded artifact; a moving branch, source checkout, or
workspace binary is not an acceptable release substitute.

An Outcome is a terminal handoff boundary, not an internal log line. Deliver a
separate visible human Outcome beginning with `Outcome: 🟢`, `Outcome: 🟡`, or
`Outcome: 🔴` and include status, unknowns, evidence, human decision, and next
action. For this Runtime, progression requires the equivalent of
`state=Verified`, `decisionState=green`, current Contract/Summary/evidence
bindings, and direct human-visible delivery. Missing, folded-only, stale,
yellow, red, contradictory, or malformed Outcome evidence fails closed.
The green Rust terminal is the reference equivalent of `status=completed` plus
`humanStatusColor=green`. The handoff must also state the issue count,
blockers or stopping reason, resolved issues, risks, verification, impact, and
next action; every factual statement needs evidence and an unproven benefit is
an inference.

When a defect is discovered during implementation, verification, finish, or
handoff, fix it in the current Work Item when its scope, authority, and base
permit the change. Amend and revalidate the Contract before adding paths or
authority, preserve retry evidence, and keep a blocked Outcome visible. Create
a successor only for a genuinely different scope, authority, or base, an
independent compatible change, an unsafe in-scope fix, immutable failed
delivery, or explicit human direction; record that reason and linkage.

After a reviewed PR passes hosted checks, close the Work Item only after the
archive and decision receipts are verified, the merged PR head SHA and
fast-forward-synchronized default branch are recorded, all relevant worktrees
are clean, and the exact remote/local Work Item branch is removed. Any failed
step is fail closed; never merge a feature branch into local `main` as a
substitute for PR review.

Keep rules language-neutral and project-neutral, never include secrets or local
credentials, and never modify user-global Agent or MCP configuration. The
reference template's `make ai-*` commands, `contractVersion: 2`, and V1
runtime assumptions are not commands or protocol requirements in this Rust
repository; use the installed Runtime lifecycle and the checks declared by the
current Contract instead.

Never revert user changes unless explicitly asked. Generated status, receipt,
and archive files are produced by the Runtime; do not hand-edit them.

The reference template's hosted-verification snapshot exception has no
equivalent command in this Rust repository. Do not push an unpublished local
snapshot as a substitute for the reviewed branch/PR workflow; use the current
Contract's declared checks and the published-artifact acceptance harness.
