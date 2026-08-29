---
author: AI Cockpit maintainers
title: Installed Runtime lifecycle
description: Installation, repository attachment, upgrade, rollback, and uninstall boundaries.
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - shared_runtime_lifecycle
---

# Installed Runtime lifecycle

[English](installed-lifecycle.md) · [简体中文](installed-lifecycle.zh-CN.md) · [日本語](installed-lifecycle.ja.md)

Installation places one shared `ai-cockpit` Runtime on the machine. It does
not attach a repository, choose a project, or prove that every repository
lifecycle is complete. Attach is explicit:

```text
ai-cockpit attach --repo /path/to/repository
ai-cockpit inspect --repo /path/to/repository
ai-cockpit doctor --repo /path/to/repository
```

The repository owns `.ai/cockpit.toml`, Contract, evidence, knowledge, and
adapter records. The Runtime has no persistent current repository or global
active Work Item.

## Release and repository boundaries

Install and upgrade must use a named, immutable public Release archive and its
SHA-256/manifest. Release distribution, Homebrew, SBOM, provenance, rollback,
and post-release adopter acceptance are documented in
[`Release and distribution`](../release/distribution.md) and remain external
to repository-local Contract decisions. A moving branch or workspace binary
is not release evidence.

Runtime-only upgrades normally leave repository bytes unchanged. A repository
schema migration is a separate, explicit, reviewed operation with a plan,
backup/rollback evidence, and a human decision. Historical evidence is not
rewritten merely because the Runtime was upgraded.

Uninstall is likewise a proposal and execution boundary: preserve repository
evidence unless the repository owner explicitly authorizes disposal. The
Runtime does not claim that an installer, provider, sandbox, or enterprise
retention system has completed an operation just because a local binary was
removed.

### Uninstall safely

Use uninstall only after the repository owner has decided to remove the
installed Runtime or its repository attachment. First perform a read-only
inventory of the AI Cockpit files that are present. Then ask whether records
must be preserved or purged, generate a removal plan without writing, and
review its affected paths, unknowns, and recovery route. Obtain a separate
confirmation before executing the plan. Execute only the approved, bounded
removal without touching unrelated project work, and verify the removal
receipt while retaining the evidence. If ownership, scope, or recovery is
unknown, stop and ask the repository owner; complete disposal is never implied
by deleting a local binary.

## Mapping to the reference source

The reference source's Python installer stages, Make targets, generated status,
and source-specific migration records are conformance material, not files to
copy. Rust uses the installed shared Runtime, explicit repository context,
typed receipts, and the public artifact acceptance harness. Any provider or
enterprise action must remain an externally verifiable evidence reference.
