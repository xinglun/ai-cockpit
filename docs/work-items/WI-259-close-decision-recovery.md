---
author: AI Cockpit maintainers
title: "WI-259 — Close decision recovery and documentation projection"
workItemId: WI-259-close-decision-recovery
description: "Recover a non-canonical predecessor close projection without rewriting immutable lifecycle records."
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-259-close-decision-recovery
authority: canonical
---

# WI-259 — Close decision recovery and documentation projection

## Intent

Preserve WI-258 exactly while recovering the documentation projection that its
close decision cannot satisfy. This successor does not reinterpret or replace
the predecessor's implementation, evidence, or human decision.

## Scope

The change is limited to the tri-language WI-258 recovery projection, the
tri-language WI-259 record, the reference-parity rows, and the Runtime-generated
WI-258 recovery decision. Production Runtime code, release artifacts, and the
predecessor `.ai` bytes are out of scope.

## Acceptance

- WI-258 archive, evidence, finalization, and close bytes remain byte-identical.
- The recovery decision binds the exact predecessor digests and successor ID.
- All three WI-258 documents and parity rows say Recovered and link this
  successor without claiming that WI-258 was rewritten.
- WI-259 is promoted to Implemented only after its own approved structured
  close and terminal evidence are present.
- Documentation, parity, governance-integrity, and promotion checks pass.

## Evidence boundary

The successor is an audit projection and recovery boundary. It does not make
the predecessor's descriptive decision equivalent to `approved`; only WI-259's
new explicit close can authorize its own terminal documentation promotion.
