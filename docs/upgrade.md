---
author: AI Cockpit maintainers
title: Upgrade
description: Compatibility entry for the Rust-native shared Runtime upgrade guide.
audience: [adopter, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-504-reference-file-comparison-batch-29
capabilityClaims:
  - runtime_upgrade
---

# Upgrade

[English](upgrade.md) · [简体中文](reference/upgrade.zh-CN.md) · [日本語](reference/upgrade.ja.md)

This compatibility entry points to the canonical [Reference Upgrade](reference/upgrade.md)
guide. It preserves the reader route used by the reference project while the
Rust-native guide explains shared Runtime installation, repository migration,
rollback, and explicit provider/Agent boundaries.

See [Release and distribution](release/distribution.md) for immutable Release
artifacts and checksums. An upgrade does not silently rewrite repository `.ai/`
state or global Agent/MCP configuration.
