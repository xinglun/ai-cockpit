---
author: AI Cockpit maintainers
workItemId: WI-1051-mac-only-prerelease-v1-0-1-rc-1
title: v1.0.1-rc.1 macOS arm64 prerelease
description: Publish one opt-in macOS arm64 prerelease for the WI-1050 Contract-amendment admission repair while preserving v1.0.0 as stable.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1051-mac-only-prerelease-v1-0-1-rc-1
---

[简体中文](WI-1051-mac-only-prerelease-v1-0-1-rc-1.zh-CN.md) · [日本語](WI-1051-mac-only-prerelease-v1-0-1-rc-1.ja.md)

# WI-1051 — v1.0.1-rc.1 macOS arm64 prerelease

This Work Item prepares and publishes one opt-in `v1.0.1-rc.1` prerelease so
the WI-1050 Contract-amendment admission repair can be tried on macOS arm64.
It is not a stable release. `v1.0.0` remains the latest stable release and
installation baseline.

## Boundaries

- The candidate source includes WI-1050 repair commit
  `09e7eead64bebacda98d71e1907c01e6008a8f88` (tree
  `3c2ac29a38ea1eb8d8e1c856424834761ab16630`).
- Publish only the macOS arm64 executable and its matching SHA-256 sidecar.
- Do not publish non-macOS assets, change the stable pointer, modify unrelated
  tags/releases, or include Task9 changes.
- Runtime formal verification, hosted CI, and non-macOS validation remain
  separate evidence boundaries and must not be implied by a local build.

## Local macOS evidence

An independent build completed with:

```text
cargo build --locked --release --package cockpit-cli --target aarch64-apple-darwin
Finished `release` profile [optimized] target(s) in 1m 31s
```

The copied isolated-prefix executable reports `ai-cockpit 1.0.1-rc.1`,
returns exit 0 for `--help`, and is an arm64 Mach-O. Its bytes match the built
artifact. The generated sidecar verifies successfully with `shasum -a 256 -c`:

```text
SHA-256: 7e15777b24480dc88c880698876a0cff03aeb5dfe019a6a68db3ecfbe9409b7f
Size: 10086320 bytes
```

These are independent local build and launch observations, not a Runtime
verification receipt. The active Runtime Summary still has no formal
verification evidence; the eight Contract scenarios remain unverified there.

## Publication and acceptance boundary

The prerelease notes must disclose any unverified Runtime review/verification,
hosted CI, non-macOS targets, and regression checks. After publication, inspect
the exact provider release, download its executable and checksum, verify the
downloaded bytes, and confirm `v1.0.0` remains the latest stable release. Keep
the Work Item in progress until its remaining evidence and lifecycle decisions
are resolved.

If withdrawal is necessary, remove only the exact `v1.0.1-rc.1` prerelease,
its matching assets, and its tag after confirming the provider identity. Never
change `v1.0.0` or unrelated releases/tags; a corrected candidate must use a
new prerelease version.
