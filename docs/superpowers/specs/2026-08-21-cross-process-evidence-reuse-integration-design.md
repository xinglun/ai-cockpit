# Cross-Process Evidence Reuse Integration Design

## Context

WI-32 closes the in-process safety boundary: a typed policy and exact composite
receipt can suppress a real process only through a fail-closed planner. The CLI
and MCP adapters still classify every project command as protected, never load
or persist reusable receipts, and never derive a repository/profile-bound
`EvidenceContext`. Therefore no reuse survives process exit and no user-visible
verification path currently claims a reuse benefit.

WI-33 integrates that proven core without installing V1, copying runtime code
into repositories, or weakening calibration and protected gates.

## Decision

### Profile-authorized reuse only

A command is `Reusable` only when the attached Project Profile is `calibrated`
and contains the exact program/arguments with state `verified`. An explicit
`--command`, detected-but-unconfirmed command, malformed profile, missing
profile digest, or unsupported stage is `NeverReuse` and executes. Security,
scope, governance, coverage, source-bound, and explicitly protected project
commands remain `Protected(class)`.

The profile digest is added explicitly to `EvidenceContext`. This evolves the
reusable receipt schema before any production receipt store exists. Older or
unknown schema receipts are Unknown and execute; they are never rewritten into
fresh evidence.

### One immutable verification context

The repository adapter derives one context per verify invocation from one
pre-execution snapshot and the confirmed profile:

- content: snapshot tree/content identity for the affected command inputs;
- diff: full base/head commit ids plus normalized changed-path digest;
- environment: canonical OS/architecture and allowlisted execution descriptor;
- command: recomputed by `cockpit-verification` from actual program, arguments,
  and working directory;
- scope/governance/policy/profile: canonical protocol and repository facts;
- toolchain: the confirmed command/toolchain descriptor;
- stage and runner: explicit local/hosted execution identity.

If the repository has no full commit identity, any required fact is missing,
or context derivation conflicts with post-command state, the candidate is
Unknown and executes.

### Bounded receipt output identity

The executor records a deterministic output digest over exit status and
bounded stdout/stderr bytes. Capture limits are explicit; truncation state is
part of the digest body. Successful reusable executions create receipts only
after the real process finishes. Failed or unspawned commands never create a
passed receipt.

### Content-addressed atomic store

Attached repositories store facts, not runtime code:

```
.ai/evidence/reuse/
  index.json
  receipts/<receipt-id>.json
```

Receipt files are immutable and named by validated receipt id. `index.json`
binds repository id, profile digest, and node id to the current receipt id. The
repository adapter writes the receipt first and atomically replaces the index;
readers load only the index and referenced receipts, avoiding a historical
directory scan. Path traversal, symlinks, repository-id mismatch, malformed
JSON, missing receipt, or index/receipt disagreement is Unknown/Execute.

The receipt/index update is post-execution evidence. A failed write cannot turn
execution into reuse and returns a truthful evidence-persistence error without
claiming a stored receipt.

### Shared CLI/MCP service boundary

Repository context derivation, profile authorization, receipt load, planning,
execution, and receipt persistence live behind one Rust service API. CLI and
MCP call that API and serialize its result; MCP does not rebuild reuse logic.
CLI/MCP results expose per-node receipt identity and the WI-32 aggregate
metrics. Protected-node skip count remains a passing invariant of zero.

## Invalidation and Dependency Rules

Any profile version/digest, command, scope, governance, content, diff,
environment, toolchain, policy, stage, runner, output identity, expiry, or
receipt/index identity change reruns the node. If a dependency reruns, WI-32
forces its downstream node to execute even when the downstream receipt is
otherwise fresh.

## Error Handling

Missing receipt storage is an empty candidate set. Malformed or untrusted
storage is reported as Unknown evidence and executes; it is not silently
repaired before execution. Profile uncertainty never enables reuse. A command
failure is a failed verification result and creates no reusable receipt.

## Verification

- First calibrated run executes and atomically persists one passed receipt;
  the unchanged second process skips the reusable node with zero spawn.
- Changing every composite/profile binding independently forces execution.
- Missing/malformed/symlinked/tampered index or receipt executes.
- Failed and truncated-output executions cannot create a passed receipt.
- Protected commands execute on both runs and protected skipped remains zero.
- CLI and MCP produce the same plan/result from the same service fixture.
- Call-count, files-read, files-hashed, git-call, and elapsed telemetry proves
  the reduction is architectural.

## Scope Boundary

WI-33 does not attach this development repository, redesign Project Profile,
add hosted provider receipts, commit, push, publish, sign, or release. Hosted
runner/provider attestation remains a later boundary. V1 stays a locked
behavioral/specification reference only.
