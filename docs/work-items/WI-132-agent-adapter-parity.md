---
author: AI Cockpit maintainers
workItemId: WI-132-agent-adapter-parity
title: Agent adapter and provider-surface parity
description: Carry the reference Contract-first and visible Outcome rules into the repository-local Agent adapter while keeping Rust Runtime boundaries explicit.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-132 — Agent adapter and provider-surface parity

## Intent

An installed Agent must receive the same safe operating boundary as the
reference source without copying its Python/Make runtime. The adapter is a
thin, explicit discovery projection; current governance state remains owned by
the shared Rust Runtime.

## Boundaries

- Use Cursor's provider-native `.cursor/rules/ai-cockpit.mdc` for new installs.
- Keep an existing managed `.cursor/rules/ai-cockpit.md` readable, owned, and
  reversible; never rename or overwrite user-owned provider files.
- Include Contract-first, unknowns, preflight human pause, Summary, visible
  Outcome, and post-merge closure guidance in the managed section.
- Expand the repository glossary and reference workflow in English, Japanese,
  and Simplified Chinese.
- Do not install provider/global configuration, change the Core protocol, or
  copy V1 runtime code, schemas, installers, Python modules, or Make commands.

## Acceptance

- Provider detection, install, doctor, repair, and detach are repository-bound,
  deterministic, isolated, and fail closed for malformed ownership or symlink
  surfaces.
- Cursor `.mdc` is canonical for new installations and managed legacy `.md`
  remains usable without an unsafe migration.
- Generated guidance requires a human pause for `not_ready` or
  `needs_human_confirmation`, preserves human-owned Contract decisions, and
  requires a visible Outcome before archive/closure.
- Glossary and tri-lingual workflow/parity documents describe the Rust
  adaptation boundary and provider-surface policy.

## Verification

See the archived Contract, verification evidence, close decision, and Runtime
evidence for focused Agent/CLI tests, workspace checks, clippy, and
documentation acceptance: `.ai/evidence/WI-132-agent-adapter-parity.verification.json`
and `.ai/decisions/WI-132-agent-adapter-parity.close.json`.
