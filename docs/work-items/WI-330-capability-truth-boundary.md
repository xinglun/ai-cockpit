---
workItemId: WI-330-capability-truth-boundary
status: in_progress
---

# WI-330 — capability-truth boundary calibration

## Intent

Compare the four pinned reference capability-claim, freshness, and Capability
Truth Matrix files against the Rust repository and record an explicit product
boundary for each one.

## File-level decision

| Pinned source path | Classification | Rust counterpart and decision |
| --- | --- | --- |
| `docs/reference/capability-claim-authoring.md` | `reference-only` | The lexical claim-binding checker is not copied. Target capability pages and the repository registry report bounded observed facts; prose metadata is not evidence. |
| `docs/reference/capability-evidence-freshness.md` | `reference-only` | Work Item receipt freshness is validated, but source Capability Truth row expiry and portable-environment policy are not target Runtime features. |
| `docs/reference/capability-truth-matrix.json` | `reference-only` | The source 30-row matrix is not a Rust wire format or authorization source. `capability_truth_registry` is request-scoped and reports observed facts and explicit exclusions. |
| `docs/reference/capability-truth-matrix.md` | `reference-only` | Target capability/adoption documentation states observed, repository, adopter, provider, and enterprise boundaries without advertising the source matrix/checker. |

## Boundary

This Work Item closes a comparison boundary; it does not add a claim checker,
row-expiry policy, new assurance level, or copied Python/V1 runtime asset. A
future Rust-native claim/evidence feature requires a separate human-owned Work
Item with schema, stale handling, multilingual scope, and adopter acceptance.

## Acceptance

- Each pinned path has a classification, counterpart, and non-overclaiming reason.
- English, Simplified Chinese, and Japanese comparison/parity records agree.
- Inventory regression covers all four paths and counts remain unchanged.
- No source Python script, source matrix JSON, or V1 runtime state is copied.

## Verification

`bash tests/conformance/reference_file_inventory_test.sh`

