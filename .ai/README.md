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
7. Discover the callable surface before guessing arguments: use
   `ai-cockpit --help` and `ai-cockpit <group> --help` for CLI commands,
   `ai-cockpit capability show --repo <repository>` for the current
   repository-bound capability registry, and `tools/list` after starting
   `ai-cockpit mcp --repo <repository>` for the typed MCP tool schemas. MCP
   calls with missing, malformed, or unknown arguments fail closed.

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

The delivery order is conditional on the Contract. A Work Item with no
external resource uses latest remote default base → dedicated branch/worktree
→ implement → preflight → checkpoint → verify → finish → archive → close →
synchronize default branch → remove its exact branch/worktree. A
resource-bound Work Item uses latest remote default base → dedicated
branch/worktree → implement → finalize-plan → preflight → checkpoint → verify
→ finish → reviewed PR → merge → declared hosted, candidate, release, or
public-artifact evidence → archive → synchronize default branch → perform
exact provider cleanup under the accepted plan → record finalize receipt →
finalize-verify → close → clean the closure/control context. In the
resource-bound route, perform the exact planned provider cleanup after merge
and required acceptance, then use `finalize` to record its identity-bound
receipt; the Runtime does not delete branches or worktrees. `finalize-verify`
validates the receipt before `close`; `close` records the terminal decision
and does not delete those resources. Cleanup after `close` refers only to the
closure/control context, never to the already finalized Work Item
branch/worktree. Do not pre-merge a feature branch into local `main`, delete
its branch before merge, or let a provider auto-delete it to bypass
finalization. If a remote step fails, preserve the retry checkout and
identity. A repository is `ready_on_base` only after the reviewed merge when
applicable, default-branch synchronization, and exact cleanup are verified; a
detached worktree is not a ready base. A historical resource-bound PR is
handled only by its exact archived Contract and valid archive manifest
through the read-only Rust gate; it is not an ordinary no-Contract route.

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
tag and downloaded binary. After a reviewed PR is merged, complete and verify
resource-bound cleanup before close; the no-resource route closes first and
cleans its exact branch and worktree afterward. Immediately after close, run
`python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all`.
If stale documentation projections are reported, complete a narrowly scoped
documentation-promotion Work Item with that helper, rerun `--check-all`, and
do not declare `ready_on_base` until it is current. Any failed step remains
open for recovery.

The repository documentation policy may declare an
`effectiveFromContractCreatedAt` timestamp. Automatic `--check-all` applies
that policy only to Contracts created at or after the timestamp; older
records remain historical and are not retroactively revalidated. An explicit
`--work-item <id> --check` still validates the requested record. For in-scope
Contracts, projection requirements follow the configured modes, operations,
scope, acceptance criteria, and existing registrations; `mode=release` alone
does not imply publication, while an explicit `release.publish` operation
does.
