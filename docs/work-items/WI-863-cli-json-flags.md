---
author: AI Cockpit maintainers
workItemId: WI-863-cli-json-flags
title: Consistent JSON flags for read-only CLI diagnostics
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-863-cli-json-flags
---

# WI-863 — Consistent JSON flags for read-only CLI diagnostics

Add the documented `--json` option to the top-level `inspect`, `status`, and
`doctor` commands. The option selects the stable machine-readable projection;
it does not change facts, exit codes, or repository scope.
