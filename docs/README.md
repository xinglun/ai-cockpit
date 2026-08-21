---
author: AI Cockpit maintainers
title: "AI Cockpit Documentation"
description: "Reader-first documentation home for understanding, adopting, and operating AI Cockpit."
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - documentation_architecture
---

# AI Cockpit documentation

[中文](README.zh-CN.md) | [日本語](README.ja.md)

This is the reader-first route through AI Cockpit. Start with the outcome you
need; the technical pages define the machine-facing contract after the user
journey is clear.

## Start here

- [Design philosophy](philosophy.md) — why evidence and human decisions are explicit.
- [Architecture](architecture.md) — runtime flow, ownership, and boundaries.
- [Capabilities and boundaries](capabilities.md) — commands, lifecycle, MCP, and recovery.
- [Release and distribution](release/distribution.md) — installation and release truth.
- [30-second command orientation](capabilities.md#capability-overview) — the current feature index.

## Choose a reader goal

| Goal | Start here | What you should be able to do |
| --- | --- | --- |
| Understand the project | [Design philosophy](philosophy.md) → [Architecture](architecture.md) | Explain the evidence flow and product boundary. |
| Decide whether to adopt | [Capabilities](capabilities.md) → [Installation](release/distribution.md) | Choose an installation path and know what it does not change. |
| Start a governed task | [Capabilities](capabilities.md#run-a-governed-work-item) → [Work Item rules](work-items/README.md) | Inspect, attach, preflight, verify, and close a bounded Work Item. |
| Create a governance skeleton | [Capabilities](capabilities.md#create-a-work-item-skeleton) → [Command reference](reference/commands.md) | Create `not_ready` scaffolding and see which human inputs remain. |
| Configure an MCP client | [Capabilities](capabilities.md#use-mcp) → [MCP distribution](release/distribution.md#mcp-and-repository-attachment) | Start the server with an explicit repository binding and read its result envelope. |
| Review or recover from a result | [Capabilities](capabilities.md#stop-and-recovery) → [Adversarial validation](security/adversarial-validation.md) | Read decisions, preserve evidence, and repair a stopped flow. |
| Maintain or audit the system | [Architecture](architecture.md) → [Protocol v1](protocol/v1/specification.md) | Find ownership, boundaries, and machine-facing contracts. |

## Technical references

- [Product boundary](architecture/product-boundary.md)
- [Runtime topology](architecture/runtime-topology.md)
- [Release distribution architecture](architecture/release-distribution.md)
- [Versioning](architecture/versioning.md)
- [Repository Protocol v1](protocol/v1/specification.md)
- [Protocol compatibility](protocol/v1/compatibility.md)
- [Performance acceptance](../tests/performance/README.md)
- [Measured performance baseline](performance/baseline.md)
- [Adversarial validation](security/adversarial-validation.md)
- [Reference](reference/README.md) — commands, configuration, and recovery.

## Maintainer and audit route

- [Work Item rules](work-items/README.md) — the governed lifecycle for this repository.
- [Command reference](reference/commands.md) — exact CLI boundaries and output.
- [Protocol v1](protocol/v1/specification.md) — repository storage and evidence contract.
- [Versioning](architecture/versioning.md) — Runtime versus repository schema boundaries.
