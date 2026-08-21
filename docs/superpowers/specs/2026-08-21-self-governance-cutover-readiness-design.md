# Self-Governance Cutover Readiness Design

## Context

This repository still uses Markdown Bootstrap Work Items. The Rust runtime has
an `attach` command, but no `.ai/` protocol state is present here. Therefore
WI-25 through WI-29 are review records, not AI Cockpit-governed lifecycle
artifacts.

## Decision

Cut over only from a reviewed, integrated, clean revision. Build the exact Rust
binary from that revision, record its identity, obtain explicit human approval,
and use `ai-cockpit attach --repo <root>` to create only repository-owned `.ai/`
facts. Never install the V1 template, runtime source, Python, schemas, scripts,
or `Makefile.ai`.

The initial profile remains `calibration_required` until a human confirms it.
Cutover is accepted only after `status` and `doctor` succeed and one disposable
governed lifecycle exercise proves start, preflight, verification, finish,
archive, and human close behavior using the central decision path.

## Preconditions

- WI-25 through WI-29 are reviewed and integrated into an identified revision.
- The worktree is clean before attach, apart from no pre-existing `.ai/` state.
- All local quality gates and the locked V1 Oracle are green on that revision.
- A human explicitly authorizes repository attach and profile confirmation.

## Safety Boundary

Opening WI-30 does not authorize commit, push, attach, profile confirmation,
hosted CI, tag, or release. Any failed precondition keeps Bootstrap governance
authoritative and must not be described as a completed installation.

## Acceptance Evidence

- Integrated revision and clean-worktree receipt.
- Runtime version and binary SHA-256 digest.
- Attach diff proving writes are confined to `.ai/` facts.
- `status` and `doctor` output with no repository runtime code.
- Human profile-confirmation decision.
- First governed lifecycle receipts and a documented rollback boundary.
