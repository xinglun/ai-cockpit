---
author: AI Cockpit maintainers
title: "Enterprise Governance Boundary"
description: "Typed authority, policy, evidence, data, retention, and audit boundaries for enterprise adopters."
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - enterprise_governance_contracts
---

# Enterprise governance boundary

AI Cockpit does not prescribe organizational structure. It enforces explicit
authority, sufficient evidence, bounded scope, visible unknowns, and auditable
decisions.

## Authority and human decisions

Authority evidence distinguishes `self_declared`, `repository_verified`,
`provider_verified`, and `enterprise_verified`. Each record names the actor,
authority source, permitted operation, policy references, and evidence
references. A human decision records the decision, actor, reason, evidence and
policy references, timestamp, and an optional recovery condition.

Approval is policy-defined, not hard-coded to two people. An organization may
choose no approval for low risk, one authorized human, multi-party approval,
or an external provider. A single responsible person can therefore be valid
when scope, fresh evidence, visible unknowns, required checks, and the decision
receipt are all explicit.

## Policy precedence

Policy is layered as organization → project → Work Item. A lower layer may add
requirements or leave an inherited rule in force, but it cannot reduce the
approval strength or remove required evidence from a higher layer. Overlay
validation fails closed when a lower policy attempts to weaken either binding.

The Runtime reads an optional strict `.ai/policy.json` document with
`schemaVersion: 1` and `organization`/`project` policy slots. A Work Item may
add a `governancePolicy` with `layer: "work_item"` in its contract. The
effective rule is selected by the explicit contract `operation` (or the
deterministic `modify_source`/`production_destructive` fallback); prose never
changes it. `preflight` reports missing authority or policy evidence,
verification refuses an already-unauthorized operation, and `finish`,
`archive`, and `close` refuse to proceed unless the effective decision is
green. A policy-protected close must use structured decision fields and bind
the policy ID in `policyRefs`. Multi-party and external-provider modes remain
fail-closed until their external approval receipt is imported.

## Delegated evidence and audit boundary

External providers remain responsible for producing their own proof. The
delegated evidence model binds provider, subject, origin, assurance, collection
time, digest, validity, and a raw evidence reference. AI Cockpit can require,
validate, display, and archive that reference; it does not manufacture a
provider signature, branch-protection result, SBOM, provenance statement, or
enterprise approval.

Use `ai-cockpit evidence import --repo <repo> --work-item <id> --metadata
<metadata.json> --raw <provider-output>` to bind provider metadata to the
digest of the exact raw bytes. The raw reference must stay under
`.ai/evidence/external/`; identical imports are idempotent and conflicting
receipts, path escapes, symlinks, unknown fields, or repository/Work Item
mismatches fail closed. `ai-cockpit evidence list` and the repository-bound MCP
`delegated_evidence_list` tool expose only revalidated receipts. Expired,
revoked, or unknown receipts remain auditable but do not satisfy a
`delegated:<provider>` evidence requirement.

Audit events carry a stable event ID, repository and Work Item identity, Runtime
identity, timestamp, digest, and evidence references. Local Git and `.ai/`
records are not claimed to be an independent immutable enterprise audit log.
Organizations requiring immutable retention should export these events to a
SIEM, WORM store, S3 Object Lock, enterprise audit system, or external ledger.

## Sensitive evidence and retention

Evidence is classified as `public`, `internal`, `confidential`, `restricted`,
or `secret_prohibited`. Persistence is selected as `full_capture`,
`redacted_capture`, `digest_only`, or `no_persistence`; secret-prohibited data
cannot use full or redacted capture. Retention metadata records expiry and a
deterministic disposal action. A retention policy may require a purge plan;
AI Cockpit does not silently delete historical evidence or claim that local
archive storage satisfies enterprise retention law.

The operational entry points are `evidence policy` and `evidence purge-plan`.
The former binds a strict policy to a Work Item; the latter returns a stable,
digest-bound list of `retain` or `purge_planned` items for an external owner to
review. No command silently removes evidence. `digest_only` stores the receipt
digest and governance summary without command output, while `no_persistence`
fails closed if the requested operation would otherwise claim completion from a
receipt that cannot be retained.

These controls support enterprise compliance work. They are not an ISO 27001,
SOC 2, or other organizational certification.
