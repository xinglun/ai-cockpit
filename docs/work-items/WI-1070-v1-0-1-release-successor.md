---
author: AI Cockpit maintainers
title: "WI-1070 — stable v1.0.1 release successor"
description: "Repairs the approved WI-1068 parity rows, adds trilingual Work Item documentation, and preserves staged v1.0.1 release acceptance."
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-1070-v1-0-1-release-successor
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1070-v1-0-1-release-successor
contractDigest: sha256:b92b2eecb57e68f7943b54f67bca249158d31c55824146c47827cb8ea43f2657
---

[简体中文](WI-1070-v1-0-1-release-successor.zh-CN.md) · [日本語](WI-1070-v1-0-1-release-successor.ja.md)

# WI-1070 — stable v1.0.1 release successor

This page is a reader-facing projection of the active [WI-1070 Contract](../../.ai/work-items/active/WI-1070-v1-0-1-release-successor.contract.json), bound to `sha256:b92b2eecb57e68f7943b54f67bca249158d31c55824146c47827cb8ea43f2657`. Runtime records remain authoritative for status, evidence, admission, and lifecycle decisions.

## Goal and stage order

The goal is accurate English, Simplified Chinese, and Japanese release references plus a stable v1.0.1 offering whose published artifacts, installations, and N-1 upgrade are verified. These are intended benefits only; report them as delivered only after their corresponding evidence passes.

The source-stage finish gate is the three-language documentation, canonical promotion check, documentation acceptance, strict source quality route, the eight-test `cognitive_benefit` integration target, and the full `cargo test --locked --workspace -- --quiet` check on the exact successor source. For the integration target, use one private immutable copy of `CARGO_BIN_EXE_ai-cockpit` for both Python and Rust while preserving the full JSON/Markdown equality and binary digest/path assertions. Run the workspace check with `RUST_TEST_THREADS=2`, `CARGO_BUILD_JOBS=1`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, one Runtime worker, and a finite explicit 900-second timeout ceiling. The Contract binds this ceiling to all five supported verification stages (`task`, `pre_ci`, `pr`, `merge`, `release`); the full-workspace command itself runs at task stage. The default timeout remains 300 seconds. After push, the exact PR head must pass its required CI and release-plan checks before PR readiness or merge. Only after normal merge and fresh Runtime publication admission may the v1.0.1 tag be created once; the existing workflow then builds and tests the candidates. Four-target candidate installation/smoke and staged Linux x86_64 upgrade from stable v1.0.0 precede public publication. Official public assets, public Linux installation/N-1, Apple Silicon macOS CLI/MCP acceptance, and finalization/close remain later gates.

## Scope and boundaries

The complete source scope is these six documentation files and the existing cognitive-benefit integration test:

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.zh-CN.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.ja.md`
- `crates/cockpit-cli/tests/cognitive_benefit.rs`

Production source and CI policy are out of scope. The only test change is the A6 private-binary isolation in `crates/cockpit-cli/tests/cognitive_benefit.rs`; make it only after fresh preflight of the current Contract and explicit human confirmation, and keep its assertions unchanged. Preserve all WI-1068 history and WI-1069 records. Do not overwrite a tag, Release, or immutable asset. Stable v1.0.0 is the N-1 predecessor; the v1.0.1 tag and Release must be unused before creation.

## Contract acceptance criteria

- **A1** Preserve the WI-1068 Contract, Summary, Outcome, events, archive, verification evidence, and prior successful or failed results byte-for-byte; append successor evidence without relabeling history.
- **A2** Repair exactly the three approved WI-1068 parity rows in English, Simplified Chinese, and Japanese with the canonical Runtime promotion projection; preserve unrelated rows.
- **A3** Add these three WI-1070 pages and the WI-1070 row in all three parity documents, bind them to the final Contract, and preserve WI-1069 records/worktree.
- **A4** Pass documentation acceptance, canonical promotion `--check-all`, and the strict quality route for the exact successor base/head, with no stale or pending parity entries.
- **A5** Resolve a fresh Runtime release plan binding PR #1022 and its exact merged source, stable v1.0.0 N-1, unused v1.0.1 tag/Release names, provider immutability metadata, and workflow gates.
- **A6** Pass the complete eight-test `cognitive_benefit` integration binary at `RUST_TEST_THREADS=2` using one private immutable copy of `CARGO_BIN_EXE_ai-cockpit` for both Python and Rust, preserving full JSON/Markdown equality and `runtimeBinaryDigest`/path assertions. Then pass `cargo test --locked --workspace -- --quiet` on the exact accepted source with `RUST_TEST_THREADS=2`, `CARGO_BUILD_JOBS=1`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, one Runtime worker, and a finite explicit 900-second timeout under this Work Item’s `modify_source` verification policy, bound to `task`, `pre_ci`, `pr`, `merge`, and `release`; this workspace check runs at task stage. An omitted timeout retains the 300-second default. Preserve all previous failures as historical evidence.
- **A7** Before PR-ready or normal merge, pass all required exact-head GitHub Actions, Rust Contract, repository-quality, package-coverage, Windows Runtime, and behavioral-oracle gates. A changed head needs fresh CI; preserve the earlier PR #1022 revision-binding failure.
- **A8** After normal merge and fresh publication admission, create the immutable v1.0.1 tag once. Before public Release publication, pass candidate install/smoke on `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`, plus staged Linux x86_64 N-1 from v1.0.0.
- **A9** After publication, verify official manifest, SHA256SUMS, and Linux x86_64 asset identity/digests; pass public install and N-1 upgrade from v1.0.0. Preserve failed downloads and never overwrite an immutable asset.
- **A10** On the local Apple Silicon host, install the official macOS asset and verify archive, manifest, checksum, installed SHA-256, version, absolute CLI path, MCP initialize, and a read-only request. Do not substitute a local build.
- **A11** Only after every required release/platform step succeeds and Runtime admits cleanup, record exact branch/worktree finalization and lifecycle receipts, then close. Keep WI-1068 history unchanged.
- **A12** Keep the multilingual documentation and validated stable v1.0.1 offering as intended benefits until corresponding documentation and publication/install/upgrade evidence passes.

The candidate, public, installation, N-1, macOS/MCP, and close checks are not complete on this page. No tag, public release, or user-visible completion is claimed here.
