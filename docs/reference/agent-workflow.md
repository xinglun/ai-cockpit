---
author: AI Cockpit maintainers
title: "Agent workflow and review boundaries"
description: "Repository-local operating rules inherited by future AI Cockpit Work Items."
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# Agent workflow and review boundaries

This page is the repository-specific projection of the applicable operating
rules from the reference source. It keeps the governance intent while using
the installed Rust Runtime and this repository's Protocol vocabulary.

## Rules inherited

- Start from the latest commit on the repository's discovered remote default
  branch. Record the remote, default branch, and base revision in the Work Item
  Contract.
- Use one Contract, one dedicated branch/worktree, and one PR per Work Item.
  Compatible independent Work Items may run concurrently when scope, evidence
  ownership, repository context, and serialized projections are isolated.
- Before editing, read `.ai/README.md` and `.ai/glossary.md`; query `inspect`,
  `status`, and `doctor`; keep edits inside the declared scope; preserve tests
  and evidence; update the Summary; and run the Contract's project checks.
- If `preflight` reports `not_ready` or `needs_human_confirmation`, pause and
  show the Preflight Review to the person. An advisory successful exit does not
  authorize implementation.
- Deliver a separate visible human Outcome with `Outcome: 🟢`, `Outcome: 🟡`,
  or `Outcome: 🔴`, unknowns, evidence, human decision, and next action. A
  missing, folded-only, stale, contradictory, or malformed Outcome fails
  closed and cannot authorize progression.
- Resolve an in-scope defect in the current Work Item by amending and
  revalidating its Contract. Create a successor only when scope, authority, or
  base genuinely differs, the change is independent, safe in-scope repair is
  impossible, immutable failed delivery requires re-delivery, or a person
  explicitly directs it.
- Installation and upgrade acceptance use an immutable published Release tag
  and downloaded binary. After merge, closure verifies archived evidence,
  decision, merged PR head, synchronized default branch, clean worktrees, and
  exact branch removal; archived evidence is validated against its immutable
  archive manifest rather than reclassified as stale solely because the merge
  changed the current repository snapshot. A failed step remains open for
  recovery.

## Project-specific adaptation

The reference source contains `make ai-*` commands and a `contractVersion: 2`
template protocol. They are not commands or schema requirements here. This
Rust project uses the installed shared Runtime and the explicit lifecycle:

```text
start → preflight → checkpoint → verify → finish → archive → close
```

Every repository-bound command carries `--repo`. The Runtime has no global
current repository, Work Item, or project profile. Contract criteria remain in
their source language; only the human presentation layer is localized.

## Safety boundary

Rules remain language-neutral and repository-local. Do not include secrets or
machine credentials, edit user-global Agent or MCP configuration, or treat
managed Agent prompts as governance authority. Do not copy V1 runtime code,
schemas, installers, or template implementation into this repository.
Never revert user changes unless explicitly asked. The default instruction read
set is `.ai/README.md`, `.ai/glossary.md`, `AGENTS.md`, and current
machine-readable governance records; `docs/archive/**` and reference material
are historical/informational unless explicitly included by a person or Contract.
Generated status, receipt, and archive files must be produced by the Runtime,
not hand-edited.
The reference template's hosted-verification snapshot exception has no
equivalent command here; never push an unpublished local snapshot as a
substitute for the reviewed branch/PR workflow.
