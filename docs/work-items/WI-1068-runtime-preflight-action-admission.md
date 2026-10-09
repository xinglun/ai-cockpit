---
author: AI Cockpit maintainers
title: "WI-1068 — Runtime preflight action admission"
description: "Keeps finish behind a green preflight bound to the current Work Item Contract and repository snapshot."
audience:
  - maintainer
  - reviewer
workItemId: WI-1068-runtime-preflight-action-admission
status: implemented
authority: historical-projection
lastVerifiedBy: WI-1068-runtime-preflight-action-admission
terminalArchive: .ai/work-items/archive/WI-1068-runtime-preflight-action-admission.contract.json
terminalVerification: .ai/evidence/WI-1068-runtime-preflight-action-admission.verification.json
terminalDecision: .ai/decisions/WI-1068-runtime-preflight-action-admission.close.json
---

# WI-1068 — Runtime preflight action admission

This page projects the archived Work Item record. The immutable records under
`.ai/work-items/archive/WI-1068-runtime-preflight-action-admission.*` remain the
source of lifecycle, verification, and close facts.

## Purpose and behavior

The Work Item aligned the Status projection with the green-preflight
requirement for `finish`. When verification evidence is valid but preflight has
not run, is not green, or is bound to an older Contract or repository
snapshot, Runtime exposes `run_preflight` and withholds `finish`. A fresh green
preflight bound to both current inputs is required before `finish` is admitted.
Existing verification, governance-control, and human-review checks remain in
force.

The implementation covered `crates/cockpit-repository/src/status_projection.rs`
and its tests, `crates/cockpit-cli/tests/collaboration_consistency.rs`, and
`.ai/policy.json`. The test fixture explicitly binds `directory.path()` as a
`&Path` before joining paths so the assessment does not report a false
`repository_material_inspection_unavailable` result.

## Archived result

The archived Contract declared `cargo fmt --all -- --check`,
`cargo test --locked --workspace`, and workspace Clippy. The archived Outcome
records verification as `verified`; its task report leaves user-visible
benefit unknown because the Work Item owner did not declare one.

[简体中文](WI-1068-runtime-preflight-action-admission.zh-CN.md) ·
[日本語](WI-1068-runtime-preflight-action-admission.ja.md)
