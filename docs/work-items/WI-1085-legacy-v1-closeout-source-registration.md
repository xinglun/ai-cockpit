---
author: AI Cockpit maintainers
title: "WI-1085 — Explicit present-day legacy v1 source registration"
description: "Present-day explicit attestation with preserved legacy history and bounded recovery."
audience:
  - maintainer
  - reviewer
workItemId: WI-1085-legacy-v1-closeout-source-registration
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1085-legacy-v1-closeout-source-registration
terminalArchive: .ai/work-items/archive/WI-1085-legacy-v1-closeout-source-registration.contract.json
terminalVerification: .ai/evidence/WI-1085-legacy-v1-closeout-source-registration.verification.json
terminalDecision: .ai/decisions/WI-1085-legacy-v1-closeout-source-registration.close.json
---

[简体中文](WI-1085-legacy-v1-closeout-source-registration.zh-CN.md) · [日本語](WI-1085-legacy-v1-closeout-source-registration.ja.md)

# WI-1085 — Explicit present-day legacy v1 source registration

This Work Item is in progress. Focused regressions and independent review are preserved; formal Runtime verification, acceptance and terminal lifecycle evidence are pending. lastVerifiedBy identifies this active projection, not a verified close.

## Intended benefit and boundary

Maintainers can explicitly register a complete legacy v1 closeout with an independent source checkout pinned to a full commit. The current registrar attests to the observed bytes; the historical producer, time, Contract, archive, verification and provider records remain unchanged. Historical evidence is not promoted to current verified execution.

## Implementation

An immutable requirement contains the exact first registration template; registration binds its complete bounded raw/canonical inventory. One Work Item lifecycle lock protects no-replace publication. Root and extension resumes require independently retained original bytes and digests. An exact root repair can remain pending while extension proof is incomplete; it does not accept an unbound transition. Lawful Retained-to-Deleted append uses the same guarded writer through explicit registration record or normal finalize.

Queries keep malformed canonical names and ASCII case aliases as obligations, bound recursive dependency work, and revoke closed/scope release when required proof is damaged. The original source checkout need not stay online after complete registration; explicit external dependencies retain their failure semantics. If all registration obligations and independently usable registration anchors disappear or roll back together, the reader cannot infer the lost registration. No remote monotonic registry is introduced.

## Evidence and remaining work

See the [implementation ledger](../../.ai/evidence/WI-1085-legacy-v1-closeout-source-registration/implementation-20261015/ledger.md) for actual RED/GREEN, compilation failures, subprocess publication tests, file bindings, resource samples and review findings. The publication crash matrix exercises the primitive; complete historical API partial-state recovery is a separate integration fixture. Neither constitutes a formal Runtime receipt.

Complete independent review corrections and three-language command/workflow documentation, obtain admission for the required parity rows, prepare material review and controls, then run declared workspace verification with one build job, isolated target and a 900-second outer bound. No PR, Ready, merge, human close, release or cleanup is authorized by this Work Item.

The all-anchor-loss boundary applies per registration generation: losing both later E and T can conceal that extension even while earlier R/G survive. An independently retained E is usable only when explicitly supplied; status does not discover external copies.
