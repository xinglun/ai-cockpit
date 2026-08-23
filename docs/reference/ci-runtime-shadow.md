---
author: AI Cockpit maintainers
title: CI Runtime verification shadow
description: Phase 1 CI convergence using an immutable public Runtime alongside existing Cargo gates.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-145-ci-runtime-shadow
---

# CI Runtime verification shadow

WI-145 establishes Phase 1 of CI convergence. The quality job downloads the
previous stable public immutable `v0.2.15` Linux Runtime, verifies its archive and binary
digests, and runs `ai-cockpit verify` against the checkout. The receipt records
the tag, version, archive digest, binary digest, platform, download source, and
the Runtime verify result.

The existing Cargo `fmt`, `clippy`, and package-test steps remain in the same
job as the independent shadow comparison. A passing Runtime shadow does not
replace or weaken those checks, and this phase does not claim result-equivalence
or provider/enterprise assurance.

The current installation baseline may advance to a newer Release (currently
`v0.2.23`) without changing this pre-publication shadow pin. The pin advances
only after that Release is public and its immutable archive/binary identity has
been recorded; this avoids a tag workflow depending on an artifact that does
not yet exist.

The convergence boundary is intentionally phased:

1. **Phase 1 (current):** immutable Runtime verify plus existing Cargo checks.
2. **Phase 2 (future):** collect comparable Runtime/Cargo results and prove
   stable convergence over time.
3. **Phase 3 (future):** remove only duplicate YAML policy after Phase 2 has
   produced evidence and a reviewed migration decision.

The shadow lane rejects source builds, workspace binaries, unpinned release
artifacts, archive/binary digest mismatches, and malformed Runtime output.
