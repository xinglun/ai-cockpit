# WI-1078 schema-v4 Retained stage

## Scope and actual state

Raydot approved only non-destructive source/sync/remote observations, a schema-v4
Retained root and the necessary evidence checkpoint on the existing recovery ref.
No Deleted transition, branch/worktree deletion, close, historical tests, Contract
amendment, tag, release, new PR or main publication occurred.

The exact executing Runtime was built from main
`bcd76ae76f61650a890267f511099c96dc6a47fd`, Rust/Cargo 1.98.1, fresh target,
single build job; SHA-256
`0c9b079a493c47552338f0bb1d5bcc0138d4025d02ecd1f495882f28e63fc1db`.
Its build provenance is retained here; binary bytes remain at the documented
local build path and were not separately published in this phase.

Actual GitHub REST reads captured HTTP 200 headers and unmodified response
bodies. Provider-verified metadata records the exact endpoint and collection
time. Source PR #1032, sync PR #1034 and exact original branch ref remain
distinct roles. Remote ref raw SHA-256 is unchanged from the old legacy import;
new observations retain the corrected singular `/git/ref/heads/` origin and
actual collection/Runtime identities without rewriting the old receipt.

## Preserved input ordering history

The first three Native observations were accepted but are not selected by the
Retained root. Their exact fractional collection times were later than Runtime's
second-resolution boundAt. A precheck discovered this before any finalize call.
All first observations/raw bytes/metadata remain unchanged.

A second actual collection was made. Before each evidence observe invocation,
the caller waited for the clock to pass the collection's second boundary. This
condition-based ordering preserves exact collection times and allows the Native
boundAt to satisfy collectedAt <= boundAt <= operation timestamp. No timestamp,
assertion, source or gate was edited. All six observations are retained, and the
root binds only the exact three second-capture selectors.

## Preserved finalization history

The first finalize call returned exit 1 before writing a root because
postMergeSync.currentMainRevision must equal the sync merge M1. That rejected
input incorrectly used the latest main tip. Its raw input/stdout/stderr are
retained. A new input corrected only this field and the actual second-invocation
timestamp. The accepted field is M1 `37cc4e540ce23a721382d9c6b510218dc5ba69c1`.
The latest separately observed main tip was `bcd76ae7`; Native validation proves
that the remote-tracking main contains M1. The actual M1 first parent remains
`508a0a0f47dcbb957aacb8b03220f6234a87549d`; provider comparison base remains
`d1a731570fb0af2ee5142e13b5d7cf9a7d4b0b50`.

Native work-item finalize then returned exit 0 and persisted the Retained root.
Native finalize-verify returned exit 1 / CleanupPending, explicitly requiring
the sequence-1 Deleted transition after Retained. This is expected at this stage
and is not a reason to rerun verification or delete resources.

Historical verification remains runtime_historical/yellow, with no invented
missing-evidence recovery gate. The original 14/14 result remains historical
truth; no revalidate-archived or verify --archived-recovery was invoked. Benefit,
intent alignment, material scanner/review and host-display limitations remain.

## Byte identities and preservation

retained-root-exact-bindings.json contains the complete selectors, raw evidence
digests, provider identities, root raw SHA-256 and a separately labeled canonical
JSON digest. The provider root's Contract digest is the archived file raw SHA-256
`97201e6fe0f59f58c72e408e5d89d387c6765fb164e9dfbe4d22895c8ff7abf6`;
status/verification canonical Contract identity remains
`e88235839caaea350e71070a0f161aaa6117680161d8fde2ae007a3686c9e63a`.
Archive raw identity remains
`b52cb7bcbc64b98a017537e4a134a24b3577d65683acea0b7017923fbeb8a836`.

All fourteen protected files (original archive set/verification and six legacy
imports), original resources and other worktrees/dirty bytes were compared and
retained. This checkpoint publishes only additions on the existing
`codex/wi1078-evidence-recovery-20261014` ref. No expanded publication scope.

## Exact machine projection

The raw post-Retained status currently reports resourceCleanup="failed" and
safeActions includes repair_finalization. The precise finalize-verify observation
is CleanupPending (Retained exists; sequence-1 Deleted absent), with the expected
exit 1. This coarse domain label is preserved verbatim, not relabeled green or
treated as a request to rerun tests. The historical human Outcome remains yellow
and has failedGate/recoveryCondition absent.
