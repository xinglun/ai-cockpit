---
author: AI Cockpit maintainers
workItemId: WI-1121-v1-0-2-release
title: v1.0.2 release
description: Prepare and verify the four-target v1.0.2 release and its public upgrade acceptance.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1121-v1-0-2-release
---

[简体中文](WI-1121-v1-0-2-release.zh-CN.md) · [日本語](WI-1121-v1-0-2-release.ja.md)

# WI-1121 — v1.0.2 release

This Work Item prepares the v1.0.2 source and distribution guidance, verifies the exact source, and then binds publication and public acceptance to an immutable annotated tag and downloaded artifacts. Publication and acceptance remain pending until their checks pass.

## Boundaries

- The planned targets are `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.
- The previous stable v1.0.1 Release is the N-1 upgrade source. Historical failed and Unknown evidence is preserved.
- A passing build alone does not establish public install, upgrade acceptance, or user-visible benefit.
