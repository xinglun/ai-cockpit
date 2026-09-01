---
author: AI Cockpit maintainers
title: "WI-464 — workflow and build rebaseline"
description: "Four source-change paths compared against the Rust-native CI and release boundaries."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
---

# WI-464 — workflow and build rebaseline

This Work Item re-reads four paths whose source bytes changed after the earlier
workflow comparison. The source checkout is the local, pinned reference at
`fde3380f81fea5fd2e288f7a8849f737dc074060`; it is a specification corpus, not
an implementation to copy.

| Pinned source path | Classification | Rust-native decision |
| --- | --- | --- |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | Source ShellCheck installation and Python/multi-stack matrix remain source/provider concerns. Rust keeps pinned-action policy, dynamic quality routing, Rust workspace/platform checks, and public adopter acceptance. |
| `.github/workflows/release.yml` | implemented-different-by-design | Source `release-digests.json` archive projection and removal of the obsolete `release.json` dual-asset check map to Rust release-manifest/`SHA256SUMS`, SBOM/provenance, platform smoke, and adopter evidence. Source projection bytes are not copied. |
| `.github/workflows/smoke.yml` | implemented-different-by-design | The source removes a `REPORT_LANGUAGE` Make argument. Rust has no source `smoke.yml`; CI, release, gate-manifest, and immutable adopter harnesses provide the bounded checks with explicit repository context. |
| `Makefile` | implemented-different-by-design | Source Python/Make shard, knowledge, and language helpers are source-only. Rust uses Cargo, the CLI, the canonical gate manifest, and explicit `--repo`; no second Make governance layer is required. |

No Rust implementation omission was found in this rebaseline. The target's
action pins remain governed by its own reviewed action-runtime policy; a source
matrix pin is not silently substituted into the Rust release/CI route.

The machine ledger records all four paths under this Work Item, preserves
`sourceChangedSincePrevious` provenance, and removes their deferred status.
The comparison is semantic/documentation parity, not source-file, Python/Make,
provider, or JSON-wire compatibility. Object/adopter repositories inherit the
Rust shared Runtime and repository-local evidence boundary, not these source
workflow files.

## Verification

- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- the declared documentation and repository gate checks for this Work Item

