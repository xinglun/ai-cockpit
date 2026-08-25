# AI Cockpit repository usage

This repository uses one externally installed `ai-cockpit` Runtime. The binary
is shared; this `.ai/` directory is private to this repository. Never infer a
current repository or Work Item from process state, the working directory, or
Agent prose.

## Agent route

1. Read-only start: `ai-cockpit inspect --repo <repository>` and
   `ai-cockpit status --repo <repository>`.
2. Confirm readiness: `ai-cockpit doctor --repo <repository>` and
   `ai-cockpit agent doctor --repo <repository> --json`.
3. For a new repository, use `ai-cockpit attach --repo <repository>`; this
   creates only AI Cockpit-owned protocol state.
4. Agent discovery is explicit: use `ai-cockpit agent list/install/repair/detach
   --repo <repository> --provider <provider>`. Do not edit global Agent or MCP
   configuration and do not treat a managed prompt as governance authority.
5. Create a skeleton with `ai-cockpit work-item new --repo <repository> --id
   <id> --mode code`. Human-owned intent, scope, acceptance, and authority must
   remain empty or unknown until a person supplies them.
6. For an authorized Work Item, use `start → preflight → checkpoint → verify →
   finish → archive → close`. Every command carries `--repo`.

The Runtime has no global active Work Item, current repository, or project
profile. Repository Protocol, Contract, evidence, knowledge, and adapter
ownership records remain isolated under this repository's `.ai/`.

## Explicit project declarations

Optional repository-owned declarations under `.ai/project/` are read-only
inputs to the Runtime projection:

- `capabilities.json` binds capabilities, non-capabilities, critical domains,
  and explicit Contract operation mappings;
- `success_criteria.json` exposes project criteria as non-authoritative
  visibility only; Contract acceptance remains the source of authority;
- `profile-policy.json` records approved boundaries, critical paths, review
  requirements, and explicit unknowns beside `.ai/project.json` identity and
  observed-quality facts.

They are strict, regular-file-only, repository- and snapshot-bound JSON. A
missing, malformed, foreign, stale, conflicting, or insufficient declaration
keeps an explicit operation in human review; intent prose and detected files
cannot satisfy a mapping. Contracts without an explicit operation retain
legacy behavior. `attach` does not invent these governance declarations.

The canonical delivery order is latest remote default base → dedicated
branch/worktree → implement → finish/archive → push → reviewed PR → merge →
close → synchronize and clean. Do not pre-merge a feature branch into local
`main`, delete its branch before merge, or let a provider auto-delete it to
bypass finalization. If a remote step fails, preserve the retry checkout and
identity. A repository is `ready_on_base` only after merge, default-branch
synchronization, and exact cleanup are verified; a detached worktree is not a
ready base.

## Evidence discipline

Do not claim `green`, `passed`, `approved`, `verified`, or `completed` from this
file. Query the Runtime and read the current repository evidence. Missing,
stale, contradictory, or unknown evidence requires a rerun, human decision, or
stop condition.

## Operating boundary inherited by future Work Items

Before editing, an Agent reads this route and `.ai/glossary.md`, queries the
Runtime with the explicit repository path, and works only inside the active
Contract's scope. The Contract records the discovered remote default branch and
base revision, human authority, acceptance criteria, required evidence, and
verification commands. Generated status, receipt, and archive files are
written by Runtime commands; tests and evidence are not removed silently.

The visible human Outcome is a terminal handoff. It must retain its
`Outcome: 🟢`, `Outcome: 🟡`, or `Outcome: 🔴` marker, unknowns, evidence,
decision, and next action. A missing, folded-only, stale, contradictory, or
malformed Outcome does not authorize finish, archive, merge, close, or release.
The green Rust terminal corresponds to the reference's `status=completed` plus
`humanStatusColor=green`: it requires `state=Verified`, `decisionState=green`,
current Contract/Summary/evidence bindings, and direct human-visible delivery.
The handoff includes issue count, blockers or stopping reason, resolved issues,
risks, verification, impact, and next action; unsupported benefits are marked
as inference. Repair an in-scope defect in the current Work Item before opening
another Work Item or Issue; a successor needs a genuinely different scope,
authority, or base, an independent change, an unsafe repair, immutable failed
delivery, or explicit human direction.
When a defect remains within the current Contract's scope, authority, and base,
amend and revalidate that Contract before creating a successor. Independent
Work Items may run concurrently only with isolated scopes, worktrees, evidence
ownership, and compatible serialized projections.

Installation and upgrade acceptance binds to an immutable published Release
tag and downloaded binary. After a reviewed PR is merged, closure verifies the
archive, decision, merged head, synchronized default branch, clean worktrees,
and exact branch removal; any failed step remains open for recovery.
