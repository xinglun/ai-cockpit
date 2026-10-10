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
contractDigest: sha256:30645ac8d81736ba66986c931042e5132685892192002fdd8cc0b9aacc21c1f8
---

[简体中文](WI-1070-v1-0-1-release-successor.zh-CN.md) · [日本語](WI-1070-v1-0-1-release-successor.ja.md)

# WI-1070 — stable v1.0.1 release successor

This page is a reader-facing projection of the active [WI-1070 Contract](../../.ai/work-items/active/WI-1070-v1-0-1-release-successor.contract.json), bound to `sha256:30645ac8d81736ba66986c931042e5132685892192002fdd8cc0b9aacc21c1f8`. Runtime records remain authoritative for status, evidence, admission, and lifecycle decisions.

## Goal and stage order

The goal is accurate English, Simplified Chinese, and Japanese release references plus a stable v1.0.1 offering whose published artifacts, installations, and N-1 upgrade are verified. These are intended benefits only; report them as delivered only after their corresponding evidence passes.

The source-stage finish gate is the three-language documentation, canonical promotion check, documentation acceptance, strict source quality route, and the declared `cargo test --locked --workspace` on the exact successor source. After push, the exact PR head must pass its required CI and release-plan checks before PR readiness or merge. Only after normal merge and fresh Runtime publication admission may candidate release checks run. Four-target candidate installation/smoke and staged Linux x86_64 upgrade from stable v1.0.0 precede public publication. Official public assets, public Linux installation/N-1, Apple Silicon macOS CLI/MCP acceptance, and finalization/close remain later gates.

## Scope and boundaries

The complete source scope is these six documentation files:

- `docs/reference/reference-parity.md`
- `docs/reference/reference-parity.zh-CN.md`
- `docs/reference/reference-parity.ja.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.zh-CN.md`
- `docs/work-items/WI-1070-v1-0-1-release-successor.ja.md`

Product source, test source, and CI policy are out of scope. Preserve all WI-1068 history and WI-1069 records. Do not overwrite a tag, Release, or immutable asset. Stable v1.0.0 is the N-1 predecessor; the v1.0.1 tag and Release must be unused before creation.

## Contract acceptance criteria

- **A1** Preserve the WI-1068 Contract, Summary, Outcome, events, archive, verification evidence, and prior successful or failed results byte-for-byte; append successor evidence without relabeling history.
- **A2** Repair exactly the three approved WI-1068 parity rows in English, Simplified Chinese, and Japanese with the canonical Runtime promotion projection; preserve unrelated rows.
- **A3** Add these three WI-1070 pages and the WI-1070 row in all three parity documents, bind them to the final Contract, and preserve WI-1069 records/worktree.
- **A4** Pass documentation acceptance, canonical promotion `--check-all`, and the strict quality route for the exact successor base/head, with no stale or pending parity entries.
- **A5** Resolve a fresh Runtime release plan binding PR #1022 and its exact merged source, stable v1.0.0 N-1, unused v1.0.1 tag/Release names, provider immutability metadata, and workflow gates.
- **A6** Pass the sole declared formal verification, `cargo test --locked --workspace`, on the exact accepted successor source; retain earlier failures as historical evidence.
- **A7** Before PR-ready or normal merge, pass all required exact-head GitHub Actions, Rust Contract, repository-quality, package-coverage, Windows Runtime, and behavioral-oracle gates. A changed head needs fresh CI; preserve the earlier PR #1022 revision-binding failure.
- **A8** After normal merge and fresh publication admission, create the immutable v1.0.1 tag once. Before public Release publication, pass candidate install/smoke on `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`, plus staged Linux x86_64 N-1 from v1.0.0.
- **A9** After publication, verify official manifest, SHA256SUMS, and Linux x86_64 asset identity/digests; pass public install and N-1 upgrade from v1.0.0. Preserve failed downloads and never overwrite an immutable asset.
- **A10** On the local Apple Silicon host, install the official macOS asset and verify archive, manifest, checksum, installed SHA-256, version, absolute CLI path, MCP initialize, and a read-only request. Do not substitute a local build.
- **A11** Only after every required release/platform step succeeds and Runtime admits cleanup, record exact branch/worktree finalization and lifecycle receipts, then close. Keep WI-1068 history unchanged.
- **A12** Keep the multilingual documentation and validated stable v1.0.1 offering as intended benefits until corresponding documentation and publication/install/upgrade evidence passes.

The candidate, public, installation, N-1, macOS/MCP, and close checks are not complete on this page. No tag, public release, or user-visible completion is claimed here.
