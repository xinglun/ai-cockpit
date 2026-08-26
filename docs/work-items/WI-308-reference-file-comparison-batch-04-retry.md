---
author: AI Cockpit maintainers
title: "WI-308 — reference evidence, trust, and rollback-corruption batch 04 retry"
workItemId: WI-308-reference-file-comparison-batch-04-retry
description: "Compare four pinned reference files and record Rust-native, adopter-readable parity boundaries."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-308-reference-file-comparison-batch-04-retry
terminalArchive: .ai/work-items/archive/WI-308-reference-file-comparison-batch-04-retry.contract.json
terminalVerification: .ai/evidence/WI-308-reference-file-comparison-batch-04-retry.verification.json
terminalFinalization: .ai/decisions/WI-308-reference-file-comparison-batch-04-retry.finalize.json
terminalDecision: .ai/decisions/WI-308-reference-file-comparison-batch-04-retry.close.json
authority: canonical
---

# WI-308 — reference evidence, trust, and rollback-corruption batch 04 retry

## Intent and goal

This Work Item compares four files from the pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`: the demo GIF,
`docs/case-study-ai-rollback-corruption.md`,
`docs/concepts/evidence-governance.md`, and `docs/concepts/trust-layer.md`.
The goal is evidence-backed, file-level parity for adopters without copying
the reference Python/Make/installer implementation or binary assets.

## File decisions

| Reference file | Classification | Target evidence and boundary |
| --- | --- | --- |
| `docs/assets/ai-cockpit-demo.gif` | reference-only | GIF89a, 800x435, 587,945 bytes, SHA-256 `88838de7221dc859efde7e8e87913d0a23a21466195647ded60612adbad1f795`; visual reference only, no binary copy. |
| `docs/case-study-ai-rollback-corruption.md` | implemented-different-by-design | Tri-language adversarial-validation docs and typed Contract/scope checks cover unauthorized paths, unrelated changes, and controlled recovery. The case is hypothetical; no automatic rollback or merge approval is claimed. |
| `docs/concepts/evidence-governance.md` | implemented-different-by-design | Enterprise governance, Outcome/evidence docs, and typed Protocol/Repository records project Evidence → Governance Decision → Human Control. Provider evidence remains delegated. |
| `docs/concepts/trust-layer.md` | implemented-different-by-design | Product boundary, philosophy, enterprise governance, and capability truth docs define calibrated trust, fail-closed unknowns, human control, and non-goals. |

The migration is semantic responsibility parity, not source wire or byte
compatibility. Contract values and evidence remain authored facts; prose is
not proof, and local evidence is not silently promoted to provider or
enterprise assurance.

## Successor and recovery boundary

The implementation was first recorded under WI-306 and its reviewed PR #268,
but that archived delivery was never merged. After WI-307 changed the default
branch parity projection, the old PR could not be updated without rewriting an
archived Contract/base or resolving a post-archive branch conflict. WI-306 is
therefore retained as immutable historical provider evidence; this successor
starts from the current remote `main` and re-establishes the same bounded file
comparison under a fresh Contract. The old PR is not treated as a current
success or failure and is not revived.

## Scope

- Update the reference inventory generator, generated ledger, and regression
  assertions for these four files.
- Record the file-level comparison in English, Chinese, and Japanese.
- Add the rollback-corruption boundary to the tri-language adversarial-
  validation docs.
- Keep the reader route and parity ledger synchronized with this Work Item.
- Validate the shared installed Runtime with an explicit repository context;
  an adopter repository must receive the same semantics while keeping its own
  `.ai/` state isolated.

## Out of scope

Rust production code, new commands or governance semantics, release/adopter or
CI changes, global Agent/MCP configuration, source Python/Make/installer code,
reference GIF or other binary copies, and immutable historical evidence/archive
bytes are outside this bounded batch.

## Acceptance and verification

1. All four pinned files are read and individually classified; the GIF digest,
   type, dimensions, and size are recorded.
2. Scope violations, unrelated changes, and completed-work rollback risk are
   explained in all three adversarial-validation documents with Rust-native
   evidence boundaries and no security overclaim.
3. Evidence Governance and Trust Layer responsibilities have explicit links to
   enterprise governance, Outcome/evidence, product boundary, philosophy, and
   capability truth material.
4. Inventory, comparison, parity, and this Work Item are synchronized; no
   WI-308 record remains deferred and no `migrate-gap` is introduced.
5. The installed `ai-cockpit` Runtime is used with explicit `--repo` through
   preflight, checkpoint, verify, finish, archive, reviewed PR, merge,
   finalization verification, close, and exact branch/worktree cleanup. The
   final human Outcome is visible in Chinese.

Required check: `cargo test --locked --workspace` plus the repository's
reference-inventory, documentation, governance-integrity, and release-quality
checks declared by the Runtime and CI.
