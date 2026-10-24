---
author: AI Cockpit maintainers
title: "WI-1107 — Complete initial plans and explicit delegated development review"
description: "Complete initial scenario plans and finite explicit agent technical review; production activation excluded."
audience:
  - maintainer
  - reviewer
workItemId: WI-1107-delegated-development-review
status: in_progress
authority: work-item-record
lastVerifiedBy: WI-1107-delegated-development-review
---

# WI-1107 — Complete initial plans and explicit delegated development review

`start --scenario-coverage <json-file>` accepts an array of the existing typed
`ScenarioCoverageEntry` values. MCP `work_item_start.scenarioCoverage` uses the
same type and repository validator. Initial entries must be unverified with
empty evidence; invalid input creates no lifecycle records. Plans are saved
before the first preflight. Omitted options preserve the legacy shape.

`start --development-review <json-file>` and MCP `developmentReview` select an
explicit Work Item policy stored at `governanceProfile.developmentReview`.
Absence remains strict. The exact baseline containing this opt-in still needs
its formal native owner preflight review. Merely claiming an actor, source or
an arbitrary approval file does not establish delegation.

```json
{"schemaVersion":1,"reviewerActor":"agent:reviewer","authoritySource":"user:explicit-bounded-delegation","allowedChanges":["append_scenarios","fill_verification_plan","append_verification"],"preapprovedCommands":["cargo test --locked"]}
```

A typed amendment may supply optional `developmentReview` AuthorityEvidence:
explicit agent/source, `assurance=self_declared`, and the sole operation
`review_development_amendment`. The finite allowlist permits only adding a
required unverified scenario with empty evidence and an exact declared or
preapproved check, filling an empty existing verification plan with such a
check, or appending an exact owner-preapproved command. Existing order and
checks remain. Scope, permissions, risk, policy, acceptance and arbitrary
command changes remain on the normal human/deny path.

The Runtime captures a separate `delegatedReview` in the native prepared and
committed amendment chain under the lifecycle lock. It binds the owner
baseline/receipt, policy, repository/WI, request/change set, before/after
Contract, source snapshot and Runtime. Missing, renamed, altered, foreign or
partial records reject. The old owner receipt is immutable, and agent review
never becomes `decisionEvidence` or discharges close, findings, failed checks,
stronger explicit human/provider policies or host/GitHub approval.

This is local provenance, not external identity authentication or protection
against an actor rewriting all local writable records. Shared Runtime and
production configuration are unchanged. Production use requires separate
explicit owner opt-in and a fresh owner review; delegated technical judgment
can be wrong. Cross-WI goal expiry/revocation and external B remain outside
this slice. A reduced number of prompts in an isolated test is not measured
real-world benefit.

Source scope: 9 production paths, 4 test paths and 12 trilingual documentation paths, plus this Work Item's native records. The governing Runtime is the fixed official v1.0.1 b2 build, not the feature being implemented. Twelve declared scenarios and nine commands require actual verification and an independent whole-branch review. Native RED/failures remain preserved; public publication, merge and true close require their own actual authorization. This source slice does not complete the full WI-1087 design.
