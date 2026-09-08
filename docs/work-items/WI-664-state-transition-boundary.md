---
author: AI Cockpit maintainers
title: "WI-664 — Lifecycle state transition boundary"
workItemId: WI-664-state-transition-boundary
description: "Make legal lifecycle transitions explicit without conflating lifecycle, evidence, governance, authority, or historical projection."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-664-state-transition-boundary
authority: canonical
---

# WI-664 — Lifecycle state transition boundary

## Intent

Make lifecycle state changes explicit and fail closed while preserving the
existing typed separation between lifecycle state, evidence, governance
decision, human authority, and historical projection.

## Before

`cockpit-core::WorkItemState` was a serializable vocabulary with no checked
transition operation. Its only direct test asserted enum equality. Callers
could therefore treat the vocabulary as descriptive and still needed to
recreate legal-transition rules elsewhere. The separate
`DecisionState`, `AuthorityState`, and `EvidenceState` types already existed,
but lifecycle movement had no pure boundary documenting that it cannot create
any of those facts.

## After

`WorkItemState::can_transition_to` defines the reviewed forward and recovery
edges, and `transition_to` returns either the requested successor or a typed
`WorkItemTransitionError`. The normal path is:

`Created → PreflightReady → ImplementationActive → VerificationPending → FinishReady → Archived → Closed`.

Paused work resumes through `ImplementationActive`; blocked or stale work
returns through `PreflightReady`, so recovery does not skip fresh checks.
Terminal states have no outgoing edge. The methods are pure and do not read
files, call Git, execute commands, persist records, infer authority, or
synthesize evidence.

## Contracts and compatibility

- Existing `snake_case` serde values, including `finish_ready` and `closed`,
  are unchanged.
- No protocol, repository, verification, file-layout, or historical-record
  format changed.
- The API is additive to `cockpit-core`; persistence and governance callers
  remain responsible for their own observations, authorization, evidence, and
  write ordering.
- Historical records are read as historical facts and are not rewritten to
  fit the internal transition vocabulary.

## P3 physical-execution audit

This Work Item does not change P3. The existing verification boundary keeps a
`PhysicalExecutionKey` independent of Work Item identity, then
`WorkItemEvidenceReceipt::bind` and `validate_for` bind a physical result to a
specific Work Item. The physical execution tests cover distinct Work Item
receipts, foreign receipt rejection, key mismatch, tampering, and foreign
execution results. A cache or shared execution result therefore does not by
itself grant a Work Item authorization or pass state.

## Verification and remaining risk

Focused `cockpit-core` tests cover forward, skipped, backward, recovery, and
terminal transitions, plus legacy serde values and unknown-value rejection.
Workspace tests, Runtime verification, governance checks, and hosted PR
checks are required before closure. The transition table is intentionally
small; it does not attempt to encode every evidence or policy combination in
one large enum. A caller must still validate current observations and
governance facts before persisting a transition.

