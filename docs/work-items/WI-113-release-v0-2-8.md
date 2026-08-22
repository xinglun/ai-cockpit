---
author: AI Cockpit maintainers
title: "WI-113 v0.2.8 public Release and self-adopter acceptance"
description: "Publish the merged Runtime, install the immutable artifact, and verify it governs this repository."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: release-adopter-acceptance
capabilityClaims:
  - public_release
  - self_adopter_acceptance
---

# WI-113: v0.2.8 public Release and self-adopter acceptance

## Goal

Publish v0.2.8 from the merged self-governed main line, install the immutable
public binary, and prove that it can govern and develop this repository without
source or workspace fallback.

## Scope

The Work Item updates workspace version metadata and current release/version
documentation, runs source and supply-chain gates, publishes the v0.2.8 tag,
installs the downloaded artifact, and records post-release adopter and N-1
acceptance evidence. Historical Work Item records and external Homebrew tap
state are not rewritten.

## Acceptance

- All workspace packages and `Cargo.lock` identify 0.2.8.
- Current English, Chinese, and Japanese release, operations, versioning, and
  parity pages identify v0.2.8; v0.2.7 remains only as the explicit N-1 input
  or historical record.
- Hosted release, artifact, manifest, checksum, provenance, and Node24 policy
  gates pass.
- The public archive and binary SHA-256 are recorded in runtime identity
  evidence; no source or workspace binary is used for adopter acceptance.
- The installed v0.2.8 Runtime reports `changedPaths=[]`, `COMPATIBLE`,
  `doctor=ok`, `agent doctor=VERIFIED`, and `runtimeCodeInRepository=false`.
- Public adopter, N-1 upgrade, isolation cleanup, evidence reuse, and
  `first-adopter-smoke=not_ready` assertions pass.
- The self-governed Work Item lifecycle and visible English/Chinese/Japanese
  Outcome handoffs are recorded before release closure.

## Verification

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features -- --test-threads=1
bash tests/docs/documentation_acceptance.sh
bash tests/release/version_consistency.sh --repo .
bash tests/release/adopter_acceptance.sh --repository xinglun/ai-cockpit --tag v0.2.8
bash tests/release/adopter_upgrade_acceptance.sh --repository xinglun/ai-cockpit --from-tag v0.2.7 --to-tag v0.2.8
```

The post-release harness is authoritative for public artifact identity and
isolation. A failed post-release result records `releasePublished: true` and
does not rewrite Release truth.

## Outcome

Status: **Implementation and release preparation complete; public publication
and downloaded-artifact acceptance are the remaining release-bound steps.**
