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
lastVerifiedBy: WI-804-release-route-ordering
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
   If the Contract has a provider/branch/worktree/PR or other resource
   finalization context, run `work-item finalize-plan` now, before verification,
   and bind the exact context. Replaying the same context is idempotent; do not
   postpone this plan until `finish` or `archive`. If no provider or external
   resource finalization applies, record that boundary and continue without
   inventing a plan.
6. Run cheap, deterministic, scope-relevant checks before expensive builds or
   integration tests. Separate source-validation inputs from governance inputs;
   do not rerun unrelated source tests merely because a receipt or status record
   changed. For a Contract that declares release or public-artifact stages, the
   default expensive-path order is: source and Contract gates; build; candidate
   fresh-install and N-1 upgrade acceptance in parallel; publish; then public
   fresh-install, public N-1 upgrade, and version consistency checks in
   parallel; archive and close only after all required evidence exists. For a
   Contract without release or public-artifact stages, stop after its declared
   source verification and lifecycle evidence; never invent a publish or
   adopter-acceptance phase. In either path, do not start a later or more
   expensive step while an earlier prerequisite is failed, unknown, or still
   pending. Stop dependent work after a prerequisite fails and emit the
   structured failure report.
7. On failure, preserve the exact output, identity, and evidence. Find and fix
   the root cause in the current Work Item when scope, authority, and base
   permit it. Retry the same operation only through a current Runtime recovery
   decision bound to the current Contract/Summary/evidence digests. A technical
   retry is not a new Work Item. Do not repeat a failed command blindly. If a
   Contract amendment follows a recovered or finish-ready state, amend first,
   then record one retry bound to the amended Contract; the older retry is
   historical and must not be reused. Use a successor only for a genuinely
   different scope, authority, base, immutable delivery, unsafe repair, or
   explicit human direction.
8. Before terminal handoff, run the declared verification and refresh all
   bindings. `finish` establishes source-verification readiness. If the
   Contract declares hosted, candidate, release, or public-artifact evidence,
   complete those declared stages before `archive`; do not ask archive to
   accept evidence that only a later stage can produce. If no later evidence is
   declared, archive after `finish`. When a resource finalization context is
   bound, record the provider receipt after `archive` and run `finalize-verify`
   before `close`; when no external resource applies, use `archive → close`
   without fabricating provider evidence. Publish, review, and merge are
   required only when the Contract declares those stages. The visible human
   Outcome must include status, unknowns, evidence, human decision, next
   action, issue count, risks, and verification. Never claim green, completed,
   merged, or released from a local pass or a folded machine record.

## CLI capability discovery

The stable command-line surface is grouped by responsibility: repository
discovery (`inspect`, `observe`, `status`, `compatibility`, `doctor`), setup and
profiles (`attach`, `migrate`, `profile`), governance and verification
(`preflight`, `gate`, `gate-plan`, `verify`), Work Item lifecycle and recovery
(`start`, `checkpoint`, `finish`, `archive`, `close`, `work-item ...`), resource
finalization (`work-item finalize-plan`, `finalize`, `finalize-verify`), evidence
and audit (`evidence ...`, `audit export`), derived knowledge (`knowledge
query`), parallel boundaries (`work-item boundary`, `work-item slot`), and Agent
orchestration (`agent first-start`, `list`, `install`, `doctor`, `repair`,
`detach`).

Before guessing a command or argument, run `ai-cockpit --help` and the relevant
`ai-cockpit <group> --help`; use `ai-cockpit capability show --repo <path>` for
the repository-bound machine-readable registry. These surfaces expose
capabilities, not authorization or readiness. Low-level Rust library helpers
such as digest functions, identity parsers, internal planners, and test or
maintenance utilities intentionally remain non-CLI APIs; use their owning
Runtime command or MCP tool instead of invoking implementation details.

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
