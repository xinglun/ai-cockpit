# Historical Finalization Adopter Recovery Design

## Problem

An older repository can contain an honest finalization receipt produced by a
previous Runtime, a shared-primary-worktree `retained` result, or a real local
merge that had no Pull Request.  Runtime 0.2.50 protects current evidence by
requiring its Runtime identity, but the resulting generic mismatch makes
historical recovery hard to discover and gives `migrate plan` no useful action.
Adopters must never edit the predecessor receipt or invent a PR number.

## Goal

Make historical finalization recovery explicit, read-only until a human binds
it, and directly usable by an adopter.  A current receipt remains strict.  A
valid closed historical receipt is projected as low-assurance historical
evidence, while a pending historical receipt remains blocked until its
append-only recovery record or complete direct-merge receipt is recorded.

## Non-goals

- Do not weaken current Runtime identity checks for active or pre-close Work
  Items.
- Do not rewrite a predecessor receipt, archive, close decision, or evidence.
- Do not invent a Pull Request, branch, worktree, authority, or human decision.
- Do not copy the reference Python/Make runtime or modify object repositories.

## Design

### Historical inventory and projection

Add a repository-bound historical finalization inventory used by read-only
`status` and `migrate plan`.  Each entry contains the Work Item identity,
predecessor path and digest when readable, Contract base, detected historical
kind, current Runtime identity, lifecycle state, assurance, and stable safe
actions.  `migrate plan` remains schema-compatible (`migrationType=none` when
the repository schema is current) but reports a separate
`historicalFinalization` collection; `status.readiness` exposes the same
collection alongside the existing pending-close list.

The inventory distinguishes:

- `historical_verified`: an immutable receipt and an existing close decision
  bind the same repository, Work Item, Contract base, and finalization digest;
- `recovery_required`: the receipt is valid but stale or legacy and the Work
  Item is not closed;
- `invalid`: malformed, foreign, stale-base, symlinked, or contradictory
  facts, always fail closed.

### Read-only recovery plan

Add `work-item finalize-recovery-plan --repo <path> --id <id>` with optional
`--kind` and `--merge-commit` hints.  It never writes repository files.  The
JSON output contains `knownFacts`, a `suggestedInput` containing only facts
deterministically derived from the repository/Git snapshot, and
`humanInputRequired`.  The suggested input uses:

- `shared_worktree_retained` for a legacy primary-worktree receipt;
- `direct_merge_no_pr` only when a complete finalization receipt can bind
  `pullRequest.number=0`, the `historical://direct-merge/<commit>` URL, the
  real merge commit, all Git parents, base, repository identity, and
  `historical_low` assurance.

The plan explicitly marks actor, authority source, reason, decision timestamp,
and any unknown resource facts as human input.  The existing
`finalize-recovery --input` remains the sole write boundary for shared-worktree
classification; complete direct-merge receipts continue through `finalize`.

### Finalize verification behavior

`finalize-verify` first validates the typed receipt, repository/Contract/base
bindings, close binding when present, and historical Git facts.  A stale
receipt with a valid immutable close is returned as a
`historical_verified` projection with `assurance=historical_low`; it is not
treated as a current failure and does not authorize a new close.  A stale
pending receipt remains nonzero and reports stable recovery actions including
the new plan command and the exact `finalize-recovery` boundary.  Current
receipts keep the existing exact Runtime identity requirement.

### Release acceptance

The release acceptance harness gains a deterministic historical fixture lane:
it downloads only the public Release binary, creates one legacy shared-worktree
receipt and one real no-PR merge fixture, verifies the plan/output and
fail-closed mutations, and records the Runtime/repository/digest/cleanup
bindings in the existing acceptance receipt.  No source checkout or workspace
binary fallback is permitted.

## Compatibility and safety

New JSON fields are optional/defaulted for old readers.  Historical projection
is informational and low assurance; it never changes `decisionState`, current
verification evidence, or authority.  All candidate files are regular
non-symlink files, identity-bound, size-bounded, and append-only.

## Verification matrix

The regression suite covers stale current-vs-historical identity, already
closed projection, shared-worktree recovery, complete direct-merge receipt,
missing/zero PR number, foreign identity, stale base, malformed JSON,
symlink, forged parents, immutable predecessor bytes, CLI plan output, and
public Release acceptance with isolated cleanup.
