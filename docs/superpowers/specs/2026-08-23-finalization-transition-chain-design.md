# Finalization transition chain design

## Problem

Resource finalization currently stores exactly one canonical `<work-item>.finalize.json`. A valid pre-merge `blocked` receipt therefore permanently prevents a later merged and cleaned observation from becoming the receipt verified by `finalize-verify` and `close`.

## Design

The canonical receipt remains the immutable chain root. Later observations are typed `ResourceFinalizationTransitionReceipt` envelopes stored at `<work-item>.finalize.<digest>.json`. Each envelope binds a zero-based successor sequence, the exact predecessor file digest, and a complete `ResourceFinalizationReceipt`.

The resolver reads the regular, non-symlink canonical root and every matching transition candidate, rejects duplicate JSON keys and malformed records, validates all repository/Work Item/Contract/resource identities, and requires one linear chain with one unique head. Missing predecessors, stale appends, forks, sequence gaps, cycles, and foreign records fail closed.

A transition preserves PR, branch, worktree, Contract, and resource-context identity. The merge commit may advance from absent to present; merged state cannot regress; deleted is terminal except for an exact replay. Historical ancestor Runtime identities remain evidence, while the appended head must bind the Runtime recording it.

`finalize-verify` validates the unique head and its local cleanup postconditions. `close` accepts only a latest `deleted` or authorized `retained` head and binds the head path and digest into its close decision. Legacy repositories containing only a canonical receipt remain valid without migration.

## WI-190 recovery topology

WI-190's canonical `unmerged/blocked` receipt remains byte-for-byte unchanged. After PR merge and resource cleanup, a current-Runtime transition records `merged/deleted` with an idempotently deleted before/after state. The resolver selects it as the unique head, enabling finalization verification and close without rewriting Release or archive truth.

## Verification

Protocol tests cover shape, identity, replay, and transition rules. Repository tests cover append, resolution, close binding, hostile candidates, local postconditions, legacy compatibility, and the WI-190 topology. CLI lifecycle tests cover user-visible recorded/appended/idempotent results. The full locked workspace suite remains required.
