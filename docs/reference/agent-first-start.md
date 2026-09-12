---
author: AI Cockpit maintainers
title: "Agent First-Start Procedure"
description: "The mandatory first-start gate projected by the installed AI Cockpit Runtime into repository-owned Agent adapters."
audience:
  - agent
  - adopter
  - maintainer
status: current
authority: canonical
capabilityClaims:
  - shared_agent_first_start_gate
---

# Agent First-Start Procedure

This procedure is projected into the repository-owned managed section when an
installed AI Cockpit Runtime runs `agent install`. It is guidance, not an
authority source: the Runtime, the active Contract, and human decisions remain
authoritative.

## Mandatory first-start gate

Complete these checks in order before changing code, tests, documentation, CI,
build files, governance files, or repository data.

1. Establish the exact repository root and use the installed Runtime with an
   explicit `--repo <path>` on every repository-bound command. Do not infer the
   repository, Work Item, Contract, or readiness from the current directory,
   Agent prose, or a previous task.
2. Read `.ai/agent-interface.json`, `.ai/README.md`, `.ai/glossary.md`, and the
   repository's current Agent instructions. Query `inspect`, `status`, and
   `doctor`; query `agent doctor` before using an adapter. If the repository is
   not attached, stop for the explicit attach decision before creating or
   changing project state.
3. Identify one active Work Item and its Contract. Confirm human-owned intent,
   scope, out-of-scope boundary, authority, acceptance criteria, required
   evidence, immutable base, and declared verification. Never invent a missing
   decision or widen scope from an error message.
4. Run `preflight` before editing. If it is `not_ready` or
   `needs_human_confirmation`, stop and show the persisted review and resume
   condition. A successful command or a yellow advisory is not authorization.
5. After the Contract is ready, record `checkpoint` before edits. Keep one
   dedicated branch and worktree per Work Item. Parallel Agents are allowed
   only for disjoint scopes with isolated worktrees/evidence and an explicit
   compatible boundary; otherwise work serially.
6. Run cheap, deterministic, scope-relevant checks before expensive builds or
   integration tests. Separate source-validation inputs from governance inputs;
   do not rerun unrelated source tests merely because a receipt or status record
   changed. Stop dependent work after a prerequisite fails and emit the
   structured failure report.
7. On failure, preserve the exact output, identity, and evidence. Find and fix
   the root cause in the current Work Item when scope, authority, and base
   permit it. Retry the same operation only through a current Runtime recovery
   decision bound to the current Contract/Summary/evidence digests. A technical
   retry is not a new Work Item. Do not repeat a failed command blindly. Use a successor only for a genuinely different
   scope, authority, base, immutable delivery, unsafe repair, or explicit human
   direction.
8. Before terminal handoff, run the declared verification and refresh all
   bindings. Use `finish → archive → close` only after the visible human Outcome
   includes status, unknowns, evidence, human decision, next action, issue
   count, risks, and verification. Never claim green, completed, merged, or
   released from a local pass or a folded machine record.

## Non-negotiable boundaries

- Do not edit global Agent or MCP configuration, secrets, credentials, or
  unrelated repository-owned files.
- Do not hand-edit Runtime-generated Contract, Summary, receipt, evidence, or
  archive files. Use the Runtime lifecycle commands.
- Preserve immutable tags, Releases, historical evidence, and retry checkouts.
  Do not overwrite a failed delivery or replace a published artifact whose
  identity does not match.
- Installation of an adapter changes only its owned managed section and
  ownership record. It does not authorize project work or imply that the
  repository's source validation, release, or public-artifact acceptance is
  complete.
