---
author: AI Cockpit maintainers
title: "Final replacement acceptance"
description: "The reproducible acceptance boundary proving the Rust Runtime replaces the reference runtime."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-82
capabilityClaims:
  - final_replacement_acceptance
---

# Final replacement acceptance

Run `tests/conformance/final_replacement_acceptance.sh --repo <repository>`
to produce an auditable acceptance directory. The harness records the
installed runtime version and binary digest, the bound repository identity,
the exact locked reference commit, each gate result, `acceptance.json`, and a
`SHA256SUMS` manifest.

The gates are deliberately separate:

- locked reference conformance;
- adversarial negative corpus;
- performance regression with a rejected negative candidate;
- release workflow policy;
- human-facing Outcome output;
- executable locked-reference oracle;
- tracked-path check proving no V1 runtime implementation was copied.

The harness fails closed. It does not call `cargo build`, `cargo run`, a
workspace binary, or a local `target/` fallback. A green receipt proves this
acceptance boundary only; it does not authorize merge or publication.
