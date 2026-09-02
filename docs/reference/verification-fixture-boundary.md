---
author: AI Cockpit maintainers
title: Verification fixture boundary
description: What an isolated repository fixture contains and what its evidence cannot prove.
audience: [contributor, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
---

# Verification fixture boundary

[简体中文](verification-fixture-boundary.zh-CN.md) · [日本語](verification-fixture-boundary.ja.md)

Repository tests may use a temporary copy to exercise the Rust Runtime. The
fixture contains source and repository-local protocol inputs, not the caller's
runtime state. Git metadata, worktrees, virtual environments, Cargo/build
outputs, and language/tool caches are excluded unless a test explicitly needs
an isolated, declared input.

Retained Work Item history is not copied merely to make a fixture smaller, and
fixture setup never deletes the source checkout. A fixture result is local test
evidence only; it is not provider, hosted-CI, adopter, production, or enterprise
evidence. Release and adopter claims require their own immutable artifact and
isolation receipts.
