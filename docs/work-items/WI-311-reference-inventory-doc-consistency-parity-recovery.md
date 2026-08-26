---
author: AI Cockpit maintainers
title: "WI-311 — reference inventory documentation parity recovery"
workItemId: WI-311-reference-inventory-doc-consistency-parity-recovery
description: "Redeliver the manifest-derived inventory counts with prearchive tri-language parity registration."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
---

# WI-311 — reference inventory documentation parity recovery

## Intent and boundary

This Work Item redelivers the bounded inventory documentation correction from
the latest `origin/main` after WI-310 was archived without the required
tri-language parity registration. The predecessor remains immutable; this
successor adds the registration before verification evidence and keeps the
reference comparison semantic rather than source-wire compatible.

## Scope

- Synchronize the three reference-file comparison ledgers with the pinned
  manifest counts (5,119 total; 4,262 generated-history; 182
  implemented-different-by-design; 1 implemented-equivalent; 3 not-applicable;
  2 reference-only; 669 deferred-next-batch; 0 migrate-gap).
- Add a deterministic regression that derives those counts and checks all three
  machine-readable markers.
- Register this Work Item in the English, Simplified Chinese, and Japanese
  reference-parity ledgers before its verification evidence is created.
- Keep the three language Work Item descriptions synchronized.

## Out of scope

Rust Runtime behavior, reference classification changes, source implementation
copies, release/adopter/CI workflow changes, global Agent/MCP configuration,
and rewriting WI-310 or any historical evidence are excluded.

## Acceptance and verification

The comparison documents, parity rows, and Work Item documents must pass the
repository documentation and governance-integrity checks. Stale, malformed,
missing, or divergent language markers must fail. The installed Runtime is
used with explicit `--repo` through the complete reviewed lifecycle; the final
human Outcome remains visible in Chinese.
