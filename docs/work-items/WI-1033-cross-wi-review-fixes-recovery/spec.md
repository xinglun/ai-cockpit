# WI-1033: Cross-WI review corrections recovery

## Purpose and lineage

This is the selected serial recovery successor to WI-1032-cross-wi-review-fixes. Runtime 0.2.113 retired WI-1032 as replaced, with verification claim not_verified, and preserved its original Contract, Summary, attempt evidence, and recovery records. WI-1033 continues the same bounded objective; it does not reinterpret the predecessor as passed.

The implementation commits are already present on this dedicated linked worktree. This Work Item supplies a complete Runtime Contract, closes the missing verification declarations, revalidates the implementation against the independent findings, and carries the selected recovery lineage through review, merge, cleanup, and historical close as Runtime permits.

The Runtime at entry is 0.2.113. The repository base is origin/main at b57561dab0893a1e4dba486305cae8ed486c61bd. No release, tag mutation, publication, or Runtime upgrade is in scope.

## Scope

- Verify the existing fixes for observed composition identity and reuse, authoritative required-check coverage, interruption recovery and lock safety, evidence containment, provider-scoped outcome selection, MCP identity parity, and query/write boundaries.
- Verify real multiple-process behavior in linked worktrees and the ordinary single-Work-Item serial route.
- Run the candidate CLI against Sentinel read-only and compare repository, Contract/evidence, lifecycle Runtime, and coordination state before and after.
- Keep every required scenario explicit, with an observable expected result and a concrete verification plan.
- Correct the repository governance-integrity gate so active recovery lineages and replaced/not_verified archives are represented without premature close or fabricated verification evidence.
- Use the repository's canonical Runtime CLI + Cargo/CI gate. The documentation command is bash tests/docs/documentation_acceptance.sh.
- Maintain accurate English, Simplified Chinese, and Japanese Work Item projections and corresponding reference-parity rows.
- Preserve the predecessor chain WI-1031 → WI-1032 → WI-1033. Resolve only those historical close obligations that the Runtime explicitly admits after current evidence is complete.

## Out of scope

- The per-commit Runtime snapshot-binding issue is the separate Task 8 Work Item requested by the user. Diagnose and implement it only after this Work Item is integrated and cleaned up; do not fold it into WI-1033.
- Changes to Sentinel source or lifecycle state, global Agent/MCP configuration, Runtime installation or upgrade, release preparation, tags, or public publication.
- Broad redesign of collaboration scheduling or storage, weakening evidence requirements, rewriting predecessor records, or claiming unverified historical work as successful.

## Acceptance criteria

1. An undeclared or incomplete node read-set disables reuse. Changing readable source or a relative executable while composition JSON remains unchanged launches the affected node.
2. Only the registered integration owner is admitted. Runtime-derived, digest-matched required checks must be complete; forged labels, missing checks, caller-supplied satisfied booleans, and an execution repository from a different Git common directory than the bound CoordinationStore are rejected before any process starts.
3. A real process killed after durable invalidation but before registration completion can be retried. Reconciliation leaves exactly one valid impact event and never evicts a live lock owner.
4. Evidence whose final file or any ancestor symlink escapes the registered worktree is rejected; outside bytes are never accepted as evidence.
5. Equal outcome identifiers from different providers do not cross-affect admission. Actions select dependencies by the exact provider/outcome pair.
6. Real CLI composition reuses identical trusted inputs without spawning a second process; when an observed input changes but JSON does not, the affected node re-runs and only unaffected trusted read-set nodes are reused.
7. MCP exposes one unambiguous identity meaning per coordination action. CLI/MCP parity and gate-manifest regression tests pass.
8. Real processes and multiple linked worktrees pass the collaboration acceptance, while ordinary single-WI verification remains serial under the canonical gate.
9. Candidate CLI inspection of Sentinel is read-only; before/after Git tree, Contract/evidence, lifecycle Runtime, and coordination bytes are identical.
10. CLI/MCP inspection, status/outcome, and dependency queries persist no coordination data. Explicit write entrances persist only their declared events, which a fresh process can consume.
11. Three-language Work Item projections and matching reference-parity rows distinguish implemented local evidence, pending Runtime verification, and verified facts accurately; the canonical documentation gate passes.
12. Independent PR review, hosted checks, merge, exact local cleanup, and the Runtime-admitted close of this selected successor lineage are handled before any release. Stop before release and return the release decision to the user.
13. A composition whose repository root is from a different Git common directory is rejected against the CoordinationStore binding, even when its copied repository ID matches, before any command starts.
14. The governance-integrity gate recognizes a valid predecessor recovery decision as an in-progress linked lineage until its selected successor is terminal; parity binds the recovery receipt and historical verification evidence without requiring a premature close receipt, then projects recovered only after successor closure.
15. A valid retired/replaced archived Work Item projects its retirement receipt and not_verified state without requiring or claiming successful verification evidence; malformed or missing retirement evidence remains fail closed.

## Required scenario expectations

| Scenario | Expected observable result |
| --- | --- |
| trusted_read_set_and_executable_identity | Missing or untrusted read-sets disable reuse; changing a readable source or the launched relative executable with unchanged JSON re-runs the affected node. |
| integration_owner_and_complete_required_checks | A non-owner or incomplete/forged required-check set is rejected before process spawn. |
| crash_reconciliation_and_live_lock_safety | A fresh process reconciles one durable event after child death, while a live lock owner remains protected. |
| registered_worktree_evidence_containment | Final and parent symlink escapes are rejected without accepting outside bytes or appending a success event. |
| provider_scoped_outcome_selection | Admission uses only the exact provider/outcome pair; a same-named outcome from another provider cannot affect it. |
| observed_input_change_reexecutes_affected_node | Exact repeat reduces composition processes from one to zero; changed observed input with unchanged JSON launches affected work and reuses only proven-unaffected nodes. |
| mcp_identity_and_query_write_boundary | CLI/MCP identities match; read queries leave durable bytes unchanged, while explicit writes are visible to a fresh process. |
| real_multi_worktree_serial_path_and_sentinel_compatibility | Linked-worktree real-process acceptance and the serial path pass; candidate Sentinel inspection changes no protected byte or lifecycle state. |
| composition_store_repository_identity_binding | A separate clone with a copied matching repository ID is rejected before spawning a composition command because its Git common directory differs from the CoordinationStore binding. |
| premerge_recovery_lineage_parity | A valid active successor is represented as an in-progress lineage with recovery and historical verification evidence, without a premature close; after successor closure the predecessor is recovered. |
| retired_work_item_not_verified_projection | A valid replaced/not_verified archive is represented with its retirement receipt and no success claim; missing or malformed retirement evidence fails closed. |

## Verification

Run the declared fmt, workspace test, clippy, gate-manifest, governance-integrity regression, release CLI build, real-process acceptance, and documentation commands from the active Contract. Runtime-bound verification occurs only after every source and documentation change is committed and the scenario preconditions are declared. A commit or changed repository snapshot requires fresh Runtime admission and evidence; no stale receipt is reused.
