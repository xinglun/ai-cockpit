---
author: AI Cockpit maintainers
title: "Capabilities and Boundaries"
description: "A reader-first overview of what the AI Cockpit runtime can do today and what remains external."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_lifecycle
  - mcp_adapter
  - agent_discovery_adapter
  - bounded_verification
  - parallel_contract_boundary
---

# Capabilities and Boundaries

## Purpose

Use this page as the current feature index. Each row describes a user-visible
operation, its starting command, and the evidence or state it produces.

## Before you start

Install or build the `ai-cockpit` binary and point it at a Git repository.
`inspect` is read-only. `attach` is the explicit recommended setup step and may
create `.ai/`; `start` can bootstrap the same protocol files when they are
missing. Review the attached profile before relying on evidence reuse.

## Terms used below

- **snapshot**: one observed repository state, including Git and relevant file digests;
- **profile**: the repository's explicitly confirmed list of known quality commands;
- **receipt**: content-bound evidence from one verification result;
- **bounded verification**: execution with a worker cap, timeout, and bounded output capture;
- **reuse**: skipping a command only when every authorized identity binding matches;
- **fail closed**: missing or contradictory evidence causes a rerun, unknown result, or stop.

## Capability overview

| Capability | What a user can do | Start here | Result |
| --- | --- | --- | --- |
| Inspect | Read repository state without changing it. | `ai-cockpit inspect --repo <path>` | Git identity, changed paths, digests, and runtime identity. |
| Attach | Create the minimum repository-owned governance scaffold. | `ai-cockpit attach --repo <path>` | `.ai/` protocol files, discovery manifest, state directories, and calibration state. |
| Compatibility and migration | Check whether an installed Runtime can safely use this repository, then apply an explicit schema migration when required. | `compatibility`, `migrate plan`, `migrate apply --approved` | `COMPATIBLE`, `MIGRATION_REQUIRED`, or `INCOMPATIBLE`; approved migrations emit a runtime-bound receipt. |
| Observe | Read the attached profile and repository facts. | `ai-cockpit observe --repo <path>` | Observation and evolution signals. |
| Preflight | Evaluate a Work Item contract before editing. | `ai-cockpit preflight --repo <path> --contract <file>` | A green, yellow, or red governance decision. |
| Work Item lifecycle | Start, checkpoint, finish, archive, and close bounded work. | `start`, `checkpoint`, `finish`, `archive`, `close` | Explicit state transitions and receipts. |
| Verification | Run allowlisted or profile-detected commands with limits. | `ai-cockpit verify --repo <path> ...` | Pass/fail/unknown result and execution evidence. |
| Evidence reuse | Avoid a repeat run only when all identity bindings match. | Confirmed profile + automatic `verify` | Reuse, or a fail-closed rerun. |
| Knowledge | Query completed repository-local evidence and explicitly materialize its derived projection. | `ai-cockpit knowledge query --repo <path>` | Filtered results plus a repository-local write boundary; never a second source of truth. |
| MCP | Expose the same repository services to an MCP client. | `ai-cockpit mcp --repo <path>` | JSON-RPC result envelopes with explicit binding. |
| Doctor | Diagnose runtime and repository readiness. | `ai-cockpit doctor --repo <path>` | Actionable diagnostics; no silent repair. |
| Profile confirmation | Confirm a quality command for controlled reuse. | `ai-cockpit profile confirm --repo <path> --program cargo --args test,--workspace` | New reviewable profile version. |
| Work Item scaffold | Create a validator-readable skeleton without inventing governance decisions. | `ai-cockpit work-item new --repo <path> --id <id> --mode <mode>` | `not_ready` Contract with snapshot-derived facts and a list of human inputs still required. |
| Profile proposal | Derive a candidate profile amendment without changing the formal baseline. | `ai-cockpit profile propose --repo <path>` | Read-only `candidate`/`proposed` output. |
| Agent adapter | Let a selected Agent host discover this repository through an owned, reversible section. | `ai-cockpit agent list/install/doctor --repo <path>` | Repository-bound discovery, ownership, state, and safe actions; no global configuration. |
| Parallel boundary and slots | Declare Contract-owned path boundaries and reserve bounded repository-local execution slots. | `ai-cockpit work-item boundary/slot ...` | Compatibility, leases, and fail-closed serialization; independent from `verify --workers`. |

## User-facing paths

### Inspect a repository

**Ask:** “Show me the repository state without changing anything.”

```bash
ai-cockpit inspect --repo /path/to/repository
```

The command reports the repository root, Git head, changed paths, tree and diff
digests, dependency fingerprint, read/hash counters, and runtime identity. If
discovery or Git fails, stop and repair the repository path before continuing.

### Attach and observe a repository

**Ask:** “Prepare this repository for governed Work Items.”

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit observe --repo /path/to/repository
```

Attachment creates the minimum repository-owned scaffold:

```text
.ai/
├── cockpit.toml
├── project.json
├── agent-interface.json
├── work-items/active/
├── work-items/archive/
├── evidence/
├── decisions/
└── knowledge/
```

It does not copy Runtime implementation or provider configuration into the target.
The first profile is `calibration_required` until a person
confirms a quality command:

```bash
ai-cockpit profile confirm --repo /path/to/repository \
  --program cargo --args test,--workspace
```

`agent-interface.json` is a repository-local discovery fact. It records the
stable repository identity and available Runtime capabilities; it is not an
Agent prompt, provider installation, authorization, or global MCP setting.

### Upgrade a Runtime or migrate a repository

Runtime upgrades and repository migrations are separate operations. A compatible
Runtime upgrade does not rewrite `.ai/` and does not create a global current
repository. Check the installed Runtime against the explicit repository first:

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
```

If the result is `MIGRATION_REQUIRED`, review the plan and explicitly approve it:

```bash
ai-cockpit migrate apply --repo /path/to/repository --approved
```

The migration receipt records the source and target schema, before/after
digests, Runtime version, and Runtime digest. It changes only the versioned
protocol files and migration record; archived Work Items, evidence, decisions,
and knowledge remain byte-for-byte historical records. `INCOMPATIBLE` stops
without a write and requires a Runtime that understands the stored schema.

When the attached protocol files are present, stateful governance operations
(`preflight`, Work Item creation/lifecycle, `verify`, knowledge/profile writes,
Agent adapter writes, and governed MCP calls) require `COMPATIBLE`. A
`MIGRATION_REQUIRED` or `INCOMPATIBLE` result stops before creating a new
record or evidence. Read-only compatibility, migration planning, observation,
status, and diagnostics remain available so an operator can review the next
safe action.

### Connect an Agent explicitly

`attach` creates the repository facts but does not modify `AGENTS.md`,
`CLAUDE.md`, `GEMINI.md`, `.cursor/`, or any home-directory configuration.
Choose a provider explicitly when you want an Agent host to discover the
repository:

```bash
ai-cockpit agent list --repo /path/to/repository
ai-cockpit agent install --repo /path/to/repository --provider codex
ai-cockpit agent doctor --repo /path/to/repository --json
```

The adapter writes only a marked section in the selected repository surface and
`.ai/adapters/<provider>.json`. It preserves unrelated bytes. `doctor` derives
`UNATTACHED`, `DISCOVERY_AVAILABLE`, `VERIFIED`, `DEGRADED`, or `CONFLICT` from
current facts; it never treats a prompt as governance authority. `repair` and
`detach` refuse modified or ambiguous sections rather than overwriting them:

```bash
ai-cockpit agent repair --repo /path/to/repository --provider codex
ai-cockpit agent detach --repo /path/to/repository --provider codex
```

Discovery, adapter installation, connection, verification, and compliance are
separate states. MCP is optional; the CLI remains usable without it, and no
provider-global configuration is changed by these commands.

The managed section points the Agent to `.ai/README.md`, the canonical
repository-local usage handoff. It requires an explicit `--repo` on every
repository-bound command and names the governed lifecycle; it does not grant
provider authorization or configure a global MCP endpoint.

### Create a Work Item skeleton

Use the scaffold when the human decision is not ready yet:

```bash
ai-cockpit work-item new --repo /path/to/repository \
  --id payment-refund-guard --mode code
```

The command fills only `repositoryId`, `baseRevision`,
`projectProfileDigest`, and `repositorySnapshotDigest`. `intent`, `scope`,
`acceptanceCriteria`, and `authority` remain empty or `unknown`; the Contract
and summary state is `not_ready`, never `passed`, `approved`, `verified`, or
`completed`. The CLI prints the known facts and the human input still needed.
The older `start` command remains available and delegates to the same scaffold
writer with explicit human fields.

Before either command creates a normal next Work Item, the Runtime evaluates a
repository-scoped entry gate. Non-`.ai` changes that predate the Contract, a
detached HEAD, a known mismatch between HEAD and the locally discovered remote
default revision, or any archived Work Item without a valid close decision is a
fail-closed stop. Archived records are never rewritten. A recovery successor
created from an identity-bound recovery decision is a continuation of the
predecessor, not an independent bypass of this gate.

Top-level `status` exposes the same read-only readiness projection under
`readiness`. `readyOnBase` is true only for a clean named branch at the
discovered default revision with no active Work Item and no pending archived
closure. Missing or ambiguous remote metadata yields `state: unknown` and
never a green claim; `blocked` and `unclosedArchivedWorkItems` identify the
exact remediation boundary.

Scaffold creation is serialized per repository and Work Item ID by a
repository-local exclusive reservation. If two `work-item new` calls race for
the same ID, exactly one creates the Contract and summary and the other fails
closed; the reservation is removed after a committed pair. Different
repositories have independent reservations and can scaffold the same ID in
parallel.

### Govern parallel Work Items

Parallel execution is Contract-bound. A boundary JSON contains the additive
`concurrencyBoundary` object with `implementationPaths`,
`generatedEvidencePaths`, `verificationOutputPaths`,
`serializedProjectionPaths`, a human reason, schema version, and `maxWorkers`.
Bind it explicitly:

```bash
ai-cockpit work-item boundary --repo /path/to/repository --id WI-123 \
  --file boundary.json
ai-cockpit work-item declare --repo /path/to/repository --id WI-123 \
  --parallelizable
ai-cockpit work-item slot acquire --repo /path/to/repository --id WI-123
ai-cockpit work-item slot list --repo /path/to/repository
ai-cockpit work-item slot release --repo /path/to/repository --id WI-123 \
  --lease-id <lease-id>
```

The existing intelligence sidecar remains the source for dependencies,
declared conflicts, and the compatibility projection. When a Contract
boundary is present, both Work Items must have explicit compatible declarations
and all four path classes are compared conservatively. Missing, malformed,
unsupported, absolute, parent, or ambiguous paths serialize execution and
cannot authorize a slot. `maxWorkers` controls these repository-local slots;
it is not `verify --workers`, which only bounds one verification run. Leases
are exclusive files under `.ai/parallel/leases/`, carry repository and Work
Item identity, and have no implicit expiry. MCP exposes the same bounded
surface as `work_item_parallel` with explicit `inspect`, `acquire`, `release`,
and `list` actions.

### Propose a profile amendment

```bash
ai-cockpit profile propose --repo /path/to/repository
```

This emits a read-only `candidate`/`proposed` amendment. It never changes the
formal `.ai/project.json` bytes or digest; a separate explicit apply decision
would be required for that change.

### Preflight a Work Item

`start` creates the contract that `preflight` reads:

```bash
ai-cockpit start --repo /path/to/repository --id WI-123 \
  --intent "Improve documentation" \
  --goal "Explain installation clearly" \
  --scope 'docs/**' --authority authorized \
  --acceptance "examples work"
ai-cockpit preflight --repo /path/to/repository \
  --contract .ai/work-items/active/WI-123.contract.json
```

Preflight evaluates the current snapshot. Missing authority, stale contract,
scope violation, or contradictory facts are stop conditions.

For an active Work Item, `preflight` also records the decision, Contract digest,
and snapshot digest in the summary. A yellow result (for example, missing
verification evidence that the next step is expected to collect) may be
checkpointed, but a red result cannot advance. Verification refreshes the
recorded decision for the resulting snapshot; `finish` requires that refresh to
be green and requires exactly one checkpoint. A checkpoint is a single serial
transition: duplicate or out-of-order lifecycle commands fail closed. Failed
checks preserve the active records so the Work Item can recover by rerunning the
missing step.

### Run a governed Work Item

**Ask:** “Start this bounded change, record progress, and close it only after review.”

```bash
# after the preflight decision is acceptable, edit only docs/**
ai-cockpit checkpoint --repo /path/to/repository --id WI-123
ai-cockpit verify --repo /path/to/repository --work-item WI-123 \
  --command cargo --args test,--workspace --workers 2
ai-cockpit finish --repo /path/to/repository --id WI-123
ai-cockpit archive --repo /path/to/repository --id WI-123
ai-cockpit close --repo /path/to/repository --id WI-123 \
  --human-decision approved
```

The expected states are `implementation_active`, `checkpointed`, `finish_ready`,
`archived`, and then `closed`. `finish` requires a passed verification receipt
for the same Work Item and current repository snapshot, a green recorded
preflight decision, and exactly one checkpoint. `archive` and `close` revalidate
the same ordered state and verification evidence even when the Contract does
not list verification in `requiredEvidenceClasses`. If a check fails, preserve
the Work Item and repair the missing evidence; do not delete its records.

`finish`, `archive`, and `close` retain the bound `outcome` object in their
stdout JSON and render the same localized human report on stderr by default.
Their `--json` mode suppresses only that stderr handoff. A blocked `finish`
renders its persisted red/yellow Outcome before preserving the original
nonzero error. Agents must surface the handoff as an explicit conversation
message; a file-only or collapsed result is not a delivery confirmation. The
CLI cannot force a host UI to expand. Hosts may surface stderr or replay
`work-item outcome`, which renders the localized report on stdout by default;
its `--json` returns the stable object. See [Human-facing
Outcome](reference/outcome-report.md).

### Verify commands and understand reuse

Explicit commands and Work Item-bound verification always execute fresh:

```bash
ai-cockpit verify --repo /path/to/repository \
  --command cargo --args test,--workspace --workers 2
```

Automatic detection can use a confirmed profile and may reuse a persisted receipt:

```bash
ai-cockpit verify --repo /path/to/repository
ai-cockpit verify --repo /path/to/repository
```

The second result may report `nodesReused: 1` and `processesSpawned: 0`. Reuse is
allowed only when repository snapshot, source/base revision, profile, toolchain,
environment, executable identity, scope, policy, stage, runner, command, and
output identity match. Protected gates, explicit commands, and Work Item runs
remain fresh. A mismatch becomes a rerun or an explicit unknown/blocked result.

Execution is bounded by a 300-second command timeout, 64 KiB per stdout/stderr
stream, and a positive worker count. Output may be marked truncated; timeout or
capture/process-tree failure is not a pass. Receipt-store index reads are capped
at 8 MiB and reusable receipts at 1 MiB; malformed, oversized, symlinked, or
inconsistent entries fail closed.

### Query knowledge and status

```bash
ai-cockpit status --repo /path/to/repository
ai-cockpit work-item status --repo /path/to/repository --id WI-123 --json
ai-cockpit work-item status --repo /path/to/repository --all --json
ai-cockpit knowledge query --repo /path/to/repository --topic installation
```

Knowledge is a projection of repository-local evidence, not a second source of
truth. An explicit `knowledge query` may create or rebuild `.ai/knowledge/`
derived indexes and reports `projection.writeBoundary=repository-local-derived`;
it never authorizes a change. Missing, stale, or invalid Work Items and receipts
must not become fresh claims. The all-Work-Item projection sorts IDs, reports
green/yellow/red/unknown counts and per-item diagnostics, and binds both the
current repository snapshot and a deterministic index digest. A malformed or
foreign member stays visible as `unknown`; it does not hide the other members or
fail open.
Repeated `observe`, `capability show`, and status projections are request-scoped
reads: they do not create tracked capability/status files or observer caches.

### Traceability, outcomes, and parallel readiness

The v2 intelligence projections keep facts separate from derived conclusions and
never invent human-owned decisions:

```bash
ai-cockpit work-item approach --repo /path/to/repository --id WI-123
ai-cockpit work-item outcome --repo /path/to/repository --id WI-123
ai-cockpit work-item inspect --repo /path/to/repository --id WI-123
ai-cockpit work-item declare --repo /path/to/repository --id WI-123 \
  --depends-on WI-100 --conflicts-with WI-124 --parallelizable
ai-cockpit knowledge query --repo /path/to/repository --v2
ai-cockpit capability show --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

`approach` emits observed facts, named derivations, evidence references, and
unknown human inputs. `outcome` distinguishes verified implementation evidence
from the human-benefit report; an undeclared benefit remains `unknown`. The
newly generated OutcomeV2 also contains a strict `taskOutcomeReport` with
evidence-bound sections, `failedGate`/`recoveryCondition`, and an append-only
`<id>.events.jsonl` source. `finish` creates the stream, `archive` binds its
digest, and `close` records the validated report as `finalReport`. Historical
records are not backfilled. The report is presentation/evidence projection,
not an approval source; full event-sourced recovery remains a separate boundary.
The capability registry separates observed technical facts from adopter-facing
Runtime claims. Adopter states are `runtime_supported`, `repository_bound`,
`observed`, `profile_confirmed`, `adopter_accepted`, `external`, and `unknown`;
the Runtime emits only the level supported by its identity, the current
snapshot, strict profile data, and repository interface. A present file is not
adopter acceptance, and `adopter_accepted` requires explicit acceptance evidence
outside this static catalog. Exclusions such as hosted CI, signing, SBOM, and
production sandboxing remain external. Missing, malformed, stale, or foreign
inputs produce stable unknowns instead of a verified claim. `inspect` fails closed for parallel execution
when dependencies, conflicts, or scope compatibility are not explicitly known.
This registry is not an installed-surface manifest: it does not reproduce the
reference template's `templateFiles`, `installedFiles`, schema/entrypoint lists,
or `verifyInstalledSurface` checks. The installer-surface manifest remains an
external Release/adopter boundary; it is not copied into the repository.

### Declare project capabilities and profile policy

Projects may add explicit, repository-owned JSON declarations under
`.ai/project/`:

- `capabilities.json` — capabilities, non-capabilities, critical domains, and
  exact `operationMappings` used by an explicit Contract operation;
- `success_criteria.json` — visible project criteria and evidence hints; it
  cannot replace Contract acceptance or create approval;
- `profile-policy.json` — approved path boundaries, critical paths, review
  requirements, and explicit unknowns. `.ai/project.json` remains the strict
  identity and observed-quality profile.

Each declaration is strict, regular-file-only, repository-identity-bound, and
bound to the reviewed repository snapshot. `capability show` and MCP
`capability_show` expose semantic digests, visible non-authoritative success
criteria, and stable unknown codes without writing declarations. When a Contract has an explicit `operation` or
`requestedOperation`, Preflight requires a matching, sufficient mapping;
missing, malformed, foreign, stale, conflicting, or insufficient declarations
remain yellow/unknown. Contract intent prose and detected files never satisfy a
mapping. Contracts without an explicit operation retain legacy behavior.

These declarations are a Rust-native governance projection, not a copy of the
reference Python runtime, Make targets, or installer manifest. `attach` does
not invent them, and project success criteria never authorize a Work Item.
Scope compatibility normalizes Windows `\\` separators, detects exact and
nested-prefix overlaps (`src/**` with `src/main.rs` or `src/test/**`), and
returns `scope_overlap_unknown` for patterns whose intersection cannot be
proven. Unknown or empty scopes are not compatible; they never authorize
parallel execution.
Diagnosis reports measured snapshot and verification cost only; it does not
pretend to be a benchmark.

Verification evidence is a strict v2 envelope. Unknown envelope fields,
malformed captured receipts, and missing nested Work Item/repository/Runtime
identity fail closed. Current CLI lifecycle commands bind evidence to the
executable's Runtime version and digest, so a foreign Runtime cannot authorize
the current Work Item. A pre-v2 evidence record is immutable historical input:
Outcome projects it as yellow `legacy_evidence_historical` and never presents it
as a current red failure or fresh green result. Re-run verification to produce
current v2 evidence.

### Use MCP

Start the server with an explicit repository binding:

```bash
ai-cockpit mcp --repo /path/to/repository
```

The server exposes these 18 tools: `status`, `work_item_get`, `work_item_outcome`, `work_item_status`, `work_item_validate`,
`work_item_list`, `blockers`, `safe_actions`, `knowledge_query`, `evidence_get`,
`delegated_evidence_list`, `repository_observe`, `capability_show`, `preflight`,
`work_item_controls`, `work_item_recover`, `verify`, and `work_item_parallel`. Use `tools/list` to inspect the
JSON-RPC schema. `preflight` requires a repository-relative `contract`; `verify`
accepts `command`, string-array `args`, and optional `workItemId`. Unbound tool
calls fail closed. Results use `structuredContent`, text content, and `isError`.
The CLI and repository-bound MCP service share the same verification policy.
`work_item_get` is a machine-oriented record lookup. `work_item_status` is a
read-only request-scoped lifecycle projection; pass `{"all": true}` for the
stable repository index. `capability_show` exposes the same Runtime-bound
registry as the CLI. For a person-facing
result, the Agent must call `work_item_outcome` with the explicit `workItemId`
and optional conversation `language`. Its text content is the same localized
human handoff rendered by the CLI, while `structuredContent.outcome` remains
the stable OutcomeV2 object. The handoff includes visible status markers,
unknowns, evidence, structured human decisions when valid, and next action.
MCP does not translate Contract source text or invent a human decision.
The human-facing projection is a presentation layer over validated OutcomeV2;
it is not a governance authority.

### Diagnose readiness

```bash
ai-cockpit doctor --repo /path/to/repository
```

Doctor reports runtime version/digest, protocol state, repository identity, and
actionable problems. It is not a generic security scanner and does not claim that
external identity, provider, branch, or production controls are satisfied.

Enterprise adopters should also read [Enterprise governance boundary](security/enterprise-governance.md)
for assurance levels, policy precedence, delegated evidence, sensitive-data persistence, retention, and
external audit export boundaries.

## What AI Cockpit does not claim

AI Cockpit is not an Agent Runtime, Workflow Engine, Security Sandbox, general
prompt-injection detector, identity provider, compliance certificate, or
replacement for human review. External identity, branch protection, production
isolation, signing, SBOM generation, provenance, and enterprise policy remain
external evidence or adopter responsibility.

## Stop and recovery

The safe response to missing or contradictory evidence is to stop, keep the Work
Item and receipt, explain the gap, repair the relevant fact, and rerun. A green
command output cannot override a red governance state.

## Next steps

1. [Installation and distribution](release/distribution.md)
2. [Architecture](architecture.md)
3. [Design philosophy](philosophy.md)
4. [Repository Protocol v1](protocol/v1/specification.md)
