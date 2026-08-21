---
author: AI Cockpit maintainers
title: "AI Cockpit"
description: "Evidence-based repository governance for AI-assisted engineering."
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_governance_layer
---

# AI Cockpit

[中文](README.zh-CN.md) | [日本語](README.ja.md)

AI Cockpit is a repository-governance runtime for AI-assisted engineering. It
turns repository facts, declared scope, verification results, and human choices
into bounded decisions that can be reviewed later.

## The problem it solves

AI-assisted changes can exceed scope, weaken tests, skip verification, or leave
reviewers without enough evidence. AI Cockpit makes the intended change, actual
repository state, required checks, unknowns, and human decision explicit.

## How it works

People and tools use the CLI or the local MCP adapter. Repository-facing state is
stored through Repository Protocol v1; the Rust governance core remains separate
from application code. The normal path is:

`inspect → attach → preflight → verify → finish/archive/close`

## Start in 30 seconds

Install the Runtime once, then attach the repository you are working in:

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
```

Read [Capabilities and boundaries](docs/capabilities.md) for the first governed
Work Item and [Release and distribution](docs/release/distribution.md) for
installation and verification.

Install the Runtime once, then attach each target repository separately:

```text
ai-cockpit attach --repo /project-a
ai-cockpit attach --repo /project-b
```

The binary is shared, but each repository keeps its own `.ai/` Contract,
Evidence, and Knowledge. Every repository-bound command requires `--repo`; the
Runtime has no global current repository or active Work Item.

`attach` creates only the minimum repository scaffold (`cockpit.toml`,
`project.json`, `agent-interface.json`, Work Item directories, evidence,
decisions, and knowledge). It does not install Agent-provider instructions.
When a task needs a governance skeleton, create one explicitly:

```bash
ai-cockpit work-item new --repo /project-a \
  --id payment-refund-guard --mode code
```

The command reports the snapshot-derived facts it could resolve and the human
inputs still required (`intent`, `scope`, `acceptanceCriteria`, and
`authority`). The result is `not_ready`; scaffolding never claims approval or
verification. `profile propose --repo /project-a` similarly emits a read-only
candidate amendment and leaves the formal profile unchanged.

To make a selected Agent host aware of the repository, use the explicit
repository-local adapter flow:

```bash
ai-cockpit agent list --repo /project-a
ai-cockpit agent install --repo /project-a --provider codex
ai-cockpit agent doctor --repo /project-a --json
```

This writes only an owned section in the selected repository surface and
`.ai/adapters/`; it never edits global Agent/MCP settings. Discovery, adapter
installation, connection, verification, and compliance remain separate states.

## Three decision states

- `green`: the required evidence supports the bounded next action;
- `yellow`: evidence is missing, stale, contradictory, or needs human confirmation;
- `red`: a required control failed or authority is absent, so the operation stops.

## Start here

- [Documentation map](docs/README.md) — choose an adopter, contributor, reviewer,
  MCP, or maintainer route.
- [Capabilities and boundaries](docs/capabilities.md) — see the current command
  surface and the responsibilities that remain external.
- [Release and distribution](docs/release/distribution.md) — installation,
  verification, rollback, and MCP configuration.

For a source checkout, contributors can inspect the command surface with
`cargo run -p cockpit-cli -- --help`. Public Release and Homebrew availability
are separate release evidence and are not implied by this checkout.

## Product boundary

This repository is not a V1 upgrade, migration, or Rust port. The V1 template is
used only as a specification source, behavioral oracle, conformance corpus, and
historical reference. Runtime code, Python modules, `Makefile.ai`, installer
files, and runtime schemas are not copied into target repositories.

AI Cockpit is not an Agent Runtime, Workflow Engine, Security Sandbox, identity
provider, compliance certificate, or replacement for human review. External
identity, branch protection, production isolation, provider Releases, and
provenance remain external evidence or adopter responsibility.
