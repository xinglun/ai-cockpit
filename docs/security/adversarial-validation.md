---
author: AI Cockpit maintainers
title: "Adversarial Validation"
description: "Fail-closed security boundaries and adversarial validation surfaces."
audience:
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - security_validation
---

# Adversarial validation

The security boundary is fail-closed and evidence-driven. The conformance
corpus is semantic rather than string-based: it compares decision state,
blockers, unknowns, safe actions, required checks, authority, and outcome state.

Runtime boundary tests additionally verify that repository text is treated as
data, Work Item IDs cannot traverse paths, MCP evidence paths stay inside the
repository, verification commands use an allowlist and target cwd, and finish
cannot self-declare completion without a fresh passed receipt.

## Verification and reuse trust boundary

Before a reusable receipt can satisfy a node, the runtime binds the candidate to
the repository snapshot and source range, attached profile/configuration bytes,
toolchain and resolved executable identity, full execution environment, command,
scope, policy, stage, runner, and output identity. Protected nodes, explicit
commands, and Work Item-bound verification execute fresh.

The receipt store rejects symlinked or malformed parents and leaves, hard-linked
commit markers, uncertain index commits, unknown schema fields, oversized files,
tampered receipt IDs, failed/expired receipts, and binding mismatches. A failure
is an unknown or rerun condition, never authorization to reuse. Verification also
caps command time, captured output, and worker count; timeout, descendant, or
capture failures are not passes.

Any failed or unknown provider result remains non-green. Human authority can
resolve a decision requirement but cannot manufacture a verification receipt.
