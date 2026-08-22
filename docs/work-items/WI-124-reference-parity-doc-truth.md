---
author: AI Cockpit maintainers
workItemId: WI-124-reference-parity-doc-truth
title: Reference parity, documentation truth, and release consistency
description: Keep the reader route, parity matrix, and operator baseline aligned with the current Runtime.
audience:
  - adopter
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-124-reference-parity-doc-truth
---

# WI-124 — Reference parity, documentation truth, and release consistency

## Intent

Make the public documentation describe the current Rust Runtime truth in
English, Simplified Chinese, and Japanese. The operator route must show the
complete governed lifecycle, the parity matrix must include the current
WI-121/WI-122/WI-123 boundaries, and release consistency checks must derive the
current baseline from Cargo metadata.

## Scope

- Root README lifecycle route and language links.
- Three reference-parity pages and their current implementation baseline.
- Three Contract/Summary field-mapping pages with explicit Rust boundaries.
- Three operations pages and version-drift checks.
- Documentation and release consistency regression scripts.
- This Work Item's three-language documentation and Runtime-generated receipts.

The field mapping is a documentation projection of the current typed Rust
Protocol; it does not introduce a new schema or claim unsupported reference
fields. Rust Runtime behavior, Agent/MCP configuration, and historical Work
Item bytes are out of scope.

## Acceptance

1. All three root READMEs show `inspect → attach → start → preflight → checkpoint → verify → finish → archive → close` and explain the gate semantics.
2. Reference parity identifies WI-121, WI-122, and WI-123 as implemented current boundaries with evidence and documentation links.
3. Contract/Summary field pages map current Rust fields and clearly label `Implemented`, `Partial`, and `External` boundaries in all three languages.
4. Operations pages describe the current adopter target without hard-coding a release number; the release script resolves the version from Cargo metadata.
5. Documentation and version consistency scripts fail when lifecycle markers, parity status, baseline target, field mapping, or stale operation versions drift.
6. No Runtime feature or global configuration is changed.

## Verification

```bash
bash tests/docs/documentation_acceptance.sh
bash tests/release/version_consistency.sh --repo .
git diff --check
```

The final human Outcome must be delivered separately with its traffic-light
marker, unknowns, evidence, human decision, and next action.
