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

`inspect → attach → start → preflight → checkpoint → verify → finish → archive → close`

`start` records the human-owned Contract, `preflight` evaluates whether work
may begin, and `checkpoint` is the serial gate before implementation proceeds.
`verify` records fresh evidence; `finish` binds the result, `archive` preserves
the immutable Work Item bundle, and `close` records the explicit human decision.

## Start in 30 seconds

Install the Runtime once, then attach the repository you are working in:

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
```

Read [Capabilities and boundaries](docs/capabilities.md) for the first governed
Work Item and [Release and distribution](docs/release/distribution.md) for
installation and verification.

## A verified complete case

The archived [WI-663 Outcome](.ai/work-items/archive/WI-663-wi659-outcome-trust-replacement.outcome.json)
is a real, bounded example of the full handoff path. It is evidence about this
repository's governance record, not a claim about universal safety or product
performance.

- **Result:** The archived record reports `state=finish_ready`,
  `decisionState=green`, and `verification.status=verified`. The separate
  [close decision](.ai/decisions/WI-663-wi659-outcome-trust-replacement.close.json)
  records the repository owner's approval; verification and approval are not
  the same fact.
- **Key changes:** The input was a presentation-layer Outcome repair on a
  declared base and bounded scope. Its recorded findings preserve distinct
  verification, lifecycle, and human-decision states, including historical,
  stale, missing, and superseded evidence.
- **Evidence boundary:** The [verification evidence](.ai/evidence/WI-663-wi659-outcome-trust-replacement.verification.json)
  supports the declared checks and repository/work-item bindings. The
  [finalization receipt](.ai/decisions/WI-663-wi659-outcome-trust-replacement.finalize.json)
  supports the recorded merge and cleanup facts. Neither file proves a release,
  universal safety, or a user-visible benefit.
- **Remaining uncertainty:** `user_visible_benefit_not_declared` remains
  explicit. When the historical record is viewed through the current Runtime,
  it may also say that historical evidence was not revalidated; that is a
  freshness limitation, not a current test failure.
- **Human next step:** No new authorization is implied by the green archived
  verification. If the evidence is needed for a current decision, revalidate
  it under the current Runtime and make the decision explicitly.

To repeat the read-only handoff lookup from a checkout, replace the placeholder
with the repository path:

```bash
repo=/path/to/ai-cockpit
ai-cockpit work-item outcome --repo "$repo" \
  --id WI-663-wi659-outcome-trust-replacement
```

The [First Work Item walkthrough](docs/getting-started/first-work-item.md)
maps the same case from input and scope through evidence, Outcome, human
decision, and cleanup.

## Shared Runtime, isolated repositories

Attach each target repository separately:

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

## What remains external

External identity, branch protection, production isolation, provider Releases,
and provenance remain external evidence or adopter responsibility. AI Cockpit
provides bounded repository governance; it does not replace human review or an
organization's security and compliance systems.
