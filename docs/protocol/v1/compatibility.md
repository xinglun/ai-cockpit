---
author: AI Cockpit maintainers
title: "Protocol Compatibility Rules"
description: "Current runtime behavior for Repository Protocol v1 compatibility."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - protocol_compatibility
---

# Protocol Compatibility Rules

These are the compatibility rules implemented by the current runtime:

1. Parse the repository protocol version without executing repository material.
2. Reject malformed or unsupported protocol versions before an operation reads or writes governed state.
3. Accept protocol major version `1` only when the consuming operation validates the fields it requires.
4. Do not silently upgrade optional capabilities or turn an unsupported request into a pass; return an
   explicit error, unknown result, or stop condition.
5. Never rewrite historical artifacts during compatibility inspection.

The current runtime supports protocol major `1`; it does not advertise a broader
minor/patch range. Runtime minor and patch releases are not migrations as long as
they preserve the v1 storage contract. A protocol-major migration creates a new
Work Item, preserves old evidence, and records the source and target protocol versions.
