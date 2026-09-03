---
author: AI Cockpit maintainers
title: "Documentation authority boundary"
description: "Reader-first documentation ownership for humans and Agents."
audience: [user, agent, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
---

# Documentation authority boundary

The canonical Agent read set is repository-local: `.ai/README.md`,
`.ai/glossary.md`, `AGENTS.md`, and the current machine-readable `.ai` records
for the bound repository. Start with `docs/current/README.md`, then use
`docs/getting-started/README.md` for adoption and `docs/reference/README.md`
for detailed commands and semantics. Language pages link to one another; a
translation is presentation, not a second policy.

Current and reference pages explain supported behavior. Historical material
under `docs/archive/**` is context only and grants no current authority unless
the human explicitly includes it in a Work Item Contract. Source-template
plans, Python scripts, Make targets, and generated reports are comparison
evidence, not instructions for this Rust repository.

Documentation checks validate frontmatter, links, locale counterparts, parity
rows, and terminal evidence. They do not silently promote a draft or infer a
governance decision. When a document describes a boundary or limitation, keep
the corresponding Runtime command, Contract field, or evidence reference
explicit; never claim that an object repository inherits a source-specific
installer, provider policy, or wire format.

Agents should query Runtime state (`inspect`, `status`, `doctor`) before acting,
use the current Work Item Contract as authority, and show a visible human
Outcome at handoff. This route is shared by every attached object repository,
while repository facts and decisions remain isolated per `--repo` context.
