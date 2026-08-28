---
author: AI Cockpit maintainers
title: "Operation-time policy re-evaluation"
description: "Fresh, fail-closed policy facts immediately before a high-risk operation."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Operation-time policy re-evaluation

[简体中文](operation-time-policy-reevaluation.zh-CN.md) · [日本語](operation-time-policy-reevaluation.ja.md)

Creation of a script, plan, or approval does not authorize its later
execution. Immediately before an executor performs a high-risk operation, an
adapter can pass a strict `OperationTimeRequest` to the Rust Core evaluator.
The request binds:

- requested operation and actual tool call;
- target resource and exact declared scope;
- prior approval operation, target, and scope;
- current attributable authority;
- evidence freshness, destructive-impact classification, and input trust.

The evaluator returns `allow`, `confirm`, or `block` facts. It never executes
the operation, writes provider resources, or grants provider permission. An
unknown operation, unclassified impact, empty scope, mismatch, stale evidence,
or non-authoritative input is never an automatic allow.

The supported high-risk vocabulary includes deletion, test/CI/branch-protection
changes, secret writes, push, merge, release, migration, script execution,
external API writes, install/upgrade, and governance uninstall. Providers and
Agents remain responsible for applying their own permission and protected
branch controls after this local evaluation.

This is a shared Runtime capability. Each adopter supplies an explicit
repository context to its surrounding command/adapter; no global current
project or approval state is created. Operation-time evaluation is a policy
input, not evidence that a provider or enterprise approved the action.
