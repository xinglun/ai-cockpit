---
author: AI Cockpit maintainers
title: "WI-186 — v0.2.23 post-release public adopter acceptance"
workItemId: WI-186-release-v0-2-23-post-release-acceptance
description: "Repository-local evidence that the immutable public v0.2.23 Runtime can govern a fresh adopter and an N-1 upgrade."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-186-release-v0-2-23-post-release-acceptance
---

# WI-186 — v0.2.23 post-release public adopter acceptance

WI-186 records the next-cycle baseline after installing the public v0.2.23
Runtime. It uses the downloaded Release binary only; it does not use a Cargo
build, `cargo run`, a workspace binary, or a local `target/` fallback.

The immutable Runtime identity is recorded in
`.ai/evidence/external/v0.2.23/adopter/runtime.json`. The public adopter run
and the v0.2.22 → v0.2.23 N-1 upgrade run retain their own `acceptance.json`,
close receipts, isolation manifests, cleanup receipts, and `SHA256SUMS`.

The adopter evidence proves that attach, Agent discovery, evidence reuse, the
`first-adopter-smoke` `not_ready` boundary, and the full Work Item lifecycle
work in an isolated repository. HOME and XDG configuration remain unchanged;
temporary and Cargo roots are explicitly classified and cleaned.

This Work Item does not rewrite a Release, tag, or historical evidence. It
records public facts so the next Work Item can use the installed v0.2.23
Runtime as its only governance interface.

Evidence: `.ai/evidence/external/v0.2.23/adopter/acceptance.json` and
`.ai/evidence/external/v0.2.23/upgrade/acceptance.json`.

[简体中文](WI-186-release-v0-2-23-post-release-acceptance.zh-CN.md) ·
[日本語](WI-186-release-v0-2-23-post-release-acceptance.ja.md)
