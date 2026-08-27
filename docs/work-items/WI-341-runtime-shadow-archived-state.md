---
author: AI Cockpit maintainers
title: "WI-341 — runtime shadow for archived pull requests"
workItemId: WI-341-runtime-shadow-archived-state
description: "Make the immutable Runtime shadow conditional on an active Contract while preserving ordinary repository gates for archived pull requests."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-341-runtime-shadow-archived-state
---

# WI-341 — runtime shadow for archived pull requests

This Work Item makes the public Runtime shadow and its artifact upload
conditional on an active Contract. An archived pull request therefore keeps
the ordinary repository gates without being falsely rejected because no active
Contract remains after `finish` and `archive`.

The change is limited to the workflow condition, its regression assertion, and
the synchronized reference documentation. It does not change Runtime Core,
release artifacts, adopter acceptance, or provider configuration.

Acceptance is recorded by the archived Contract and verification evidence;
the reviewed pull request remains the provider-side boundary until close.

[简体中文](WI-341-runtime-shadow-archived-state.zh-CN.md) ·
[日本語](WI-341-runtime-shadow-archived-state.ja.md)
