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
| Knowledge | Query completed repository-local evidence. | `ai-cockpit knowledge query --repo <path>` | Filtered results; never a second source of truth. |
| MCP | Expose the same repository services to an MCP client. | `ai-cockpit mcp --repo <path>` | JSON-RPC result envelopes with explicit binding. |
| Doctor | Diagnose runtime and repository readiness. | `ai-cockpit doctor --repo <path>` | Actionable diagnostics; no silent repair. |
| Profile confirmation | Confirm a quality command for controlled reuse. | `ai-cockpit profile confirm --repo <path> --program cargo --args test,--workspace` | New reviewable profile version. |
| Work Item scaffold | Create a validator-readable skeleton without inventing governance decisions. | `ai-cockpit work-item new --repo <path> --id <id> --mode <mode>` | `not_ready` Contract with snapshot-derived facts and a list of human inputs still required. |
| Profile proposal | Derive a candidate profile amendment without changing the formal baseline. | `ai-cockpit profile propose --repo <path>` | Read-only `candidate`/`proposed` output. |
| Agent adapter | Let a selected Agent host discover this repository through an owned, reversible section. | `ai-cockpit agent list/install/doctor --repo <path>` | Repository-bound discovery, ownership, state, and safe actions; no global configuration. |

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

It does not copy Rust source, V1 runtime files, Python helpers, provider
instructions, or runtime schemas into the target. The first profile is `calibration_required` until a person
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

Scaffold creation is serialized per repository and Work Item ID by a
repository-local exclusive reservation. If two `work-item new` calls race for
the same ID, exactly one creates the Contract and summary and the other fails
closed; the reservation is removed after a committed pair. Different
repositories have independent reservations and can scaffold the same ID in
parallel.

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
for the same Work Item and current repository snapshot. `close` requires the
archive manifest and a human decision. If a check fails, preserve the Work Item
and repair the missing evidence; do not delete its records.

`finish`, `archive`, and `close` each emit the bound `outcome` object in their
JSON result. Agents must surface that Outcome as an explicit conversation
message; a file-only or collapsed result is not a delivery confirmation.
For a readable handoff, `work-item outcome` renders the localized human report
by default; use `--json` when an Agent or script needs the stable object. See
[Human-facing Outcome](reference/outcome-report.md).

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
ai-cockpit knowledge query --repo /path/to/repository --topic installation
```

Knowledge is a projection of repository-local evidence, not a second source of
truth. Missing, stale, or invalid Work Items and receipts must not become fresh claims.

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
capability registry reports detection versus profile-confirmed verification and
includes confidence and evidence. `inspect` fails closed for parallel execution
when dependencies, conflicts, or scope compatibility are not explicitly known.
Diagnosis reports measured snapshot and verification cost only; it does not
pretend to be a benchmark.

### Use MCP

Start the server with an explicit repository binding:

```bash
ai-cockpit mcp --repo /path/to/repository
```

The server exposes these tools: `status`, `work_item_get`, `work_item_list`,
`blockers`, `safe_actions`, `knowledge_query`, `evidence_get`,
`repository_observe`, `preflight`, and `verify`. Use `tools/list` to inspect the
JSON-RPC schema. `preflight` requires a repository-relative `contract`; `verify`
accepts `command`, string-array `args`, and optional `workItemId`. Unbound tool
calls fail closed. Results use `structuredContent`, text content, and `isError`.
The CLI and repository-bound MCP service share the same verification policy.

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
