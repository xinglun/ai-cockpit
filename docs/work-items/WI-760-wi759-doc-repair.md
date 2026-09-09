---
author: AI Cockpit maintainers
title: "WI-760 — WI-759 documentation repair successor"
description: "Supply the missing self-projection pages for WI-759 and bind the successor documentation boundary."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorization
workItemId: WI-760-wi759-doc-repair
lastVerifiedBy: WI-760-wi759-doc-repair
---

[简体中文](WI-760-wi759-doc-repair.zh-CN.md) · [日本語](WI-760-wi759-doc-repair.ja.md)

# WI-760 — WI-759 documentation repair successor

## Intent

Repair the missing self-projection pages discovered after WI-759's reviewed PR
#742 merged. The successor preserves WI-759's immutable evidence and makes the
pending documentation boundary explicit.

## Boundary

This Work Item changes only the named three-language documentation pages and
reference-parity projections. It does not change Runtime behavior, product
code, machine contracts, authorization semantics, exit codes, or historical
WI-759 evidence.

## Acceptance

- WI-759 and WI-760 each have accurate three-language pages.
- Parity rows bind local Work Item pages and the relevant immutable evidence.
- The pre-archive documentation, governance, and status checks pass without
  claiming a terminal close before Runtime generates it.
