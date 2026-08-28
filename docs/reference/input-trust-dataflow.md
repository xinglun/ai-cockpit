---
author: AI Cockpit maintainers
title: Input trust data flow
description: Provenance-aware handling of repository content, tool output, and generated interpretations.
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - provenance_aware_observation
---

# Input trust data flow

[English](input-trust-dataflow.md) · [简体中文](input-trust-dataflow.zh-CN.md) · [日本語](input-trust-dataflow.ja.md)

AI Cockpit treats repository content and tool output as inputs to be classified,
not as authority. A command-looking line in Markdown, an issue role claim, or
an agent-generated conclusion does not become permission or independent
evidence merely because it was observed.

## Rust-native provenance

The Runtime represents bounded provenance with typed `FactOrigin`,
`TraceableFact`, and `TraceableDerivation` records. Typical origins are
`Observed`, `Declared`, `Derived`, `External`, and `Unknown`. Snapshot facts,
build detection, test output, and repository documents remain traceable to the
repository and operation that produced them; derived signals retain their
input references and rule.

This is semantic parity with the reference trust data flow, not source JSON
wire compatibility. The target does not copy the reference Python trust
module or invent provider authentication.

## Safe handling rules

- Direct user instructions and repository policy can be authority for the
  bounded operation; repository documents, issues, PRs, web pages, fixtures,
  and logs are content or untrusted observations.
- Tool output is data. An agent interpretation of that output is not a new
  independent verification result.
- Cross-step use preserves the original origin and appends the new derivation;
  a later step cannot erase an earlier unknown or untrusted source.
- Missing provenance, contradictory identity, unsafe instruction injection,
  or an unknown/generated conclusion at a high-risk boundary stops the local
  action and exposes a safe alternative or human review requirement.

The trust layer does not authenticate a person, verify a provider, or authorize
an external merge/release. Those responsibilities remain with the explicit
human decision, provider, or enterprise evidence boundary.

## Object repositories

An adopter receives the same fail-closed classification rules through its
attached Runtime, while its facts and evidence remain repository-local. Every
Runtime call still requires an explicit `--repo`; there is no global current
project or shared provenance state.
