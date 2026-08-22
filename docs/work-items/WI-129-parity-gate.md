---
author: AI Cockpit maintainers
workItemId: WI-129-parity-gate
title: Enforce reference parity completeness
description: Make the documentation gate derive the latest implemented Work Item instead of relying only on a fixed list.
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-129 — Reference parity completeness

The three-language parity baseline now includes WI-128. The documentation
acceptance gate also derives the highest numeric Work Item ID whose canonical
English document is marked `status: implemented` and requires that ID in every
parity language. This makes a newly merged implementation omission fail closed
instead of depending on a forgotten fixed-list edit.

The gate remains read-only and does not infer governance facts or mutate
Runtime, Contract, Summary, or evidence state.
