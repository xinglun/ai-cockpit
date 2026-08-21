---
author: AI Cockpit maintainers
title: "Threat Model"
description: "Trust assumptions, protected assets, and fail-closed threats for the shared Runtime and repository Protocol."
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - threat_model
---

# Threat model

## Assets

The protected assets are repository identity, Work Item scope and authority,
verification outputs, reusable receipts, archived history, Runtime identity,
and Agent adapter ownership. `.ai/` is repository-local state; the installed
Runtime is shared code and must not create a global current repository.

## Trust boundaries

- Human requests and Work Item Contracts are declared inputs, not proof.
- Repository files, logs, dependency instructions, and provider messages are
  untrusted material until represented as typed facts.
- Verification commands execute within bounded Runtime controls, but the
  Runtime does not claim to be a general operating-system sandbox.
- External CI, identity, signatures, SBOM, provenance, SIEM, WORM storage, and
  enterprise approvals remain external evidence producers or retention owners.

## Threats and responses

Scope expansion, missing authority, stale or cross-Work Item evidence,
repository/log prompt injection, test weakening, unsafe deletion, receipt
tampering, path traversal, symlinks, oversized store data, and executable
identity drift fail closed or require a fresh run. Wording alone cannot grant a
capability; Raw Request Binding must declare operation, scope, authority, and
evidence facts.

The model does not claim to detect every malicious intention. It proves only
the deterministic boundaries represented by the request and evidence schema.
